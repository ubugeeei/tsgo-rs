use crate::Result;
use crate::api::{ApiClient, ApiProfile, ManagedSnapshot, UpdateSnapshotParams};
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    sync::mpsc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tsgo_rs_core::fast::{CompactString, FastMap, SmallVec};
use tsgo_rs_runtime::block_on;

/// Local pool/cache orchestrator for multiple tsgo API workers.
#[derive(Default)]
pub struct ApiOrchestrator {
    fleets: RwLock<FastMap<CompactString, Arc<ClientFleet>>>,
    snapshots: RwLock<FastMap<CompactString, Arc<ManagedSnapshot>>>,
    cached: RwLock<FastMap<CompactString, CachedValue>>,
}

struct ClientFleet {
    next: AtomicUsize,
    clients: RwLock<SmallVec<[ApiClient; 4]>>,
}

struct CachedValue {
    expires_at: Option<Instant>,
    bytes: SmallVec<[u8; 256]>,
}

impl ApiOrchestrator {
    /// Ensures that at least `replicas` workers have been started for `profile`.
    pub async fn prewarm(&self, profile: &ApiProfile, replicas: usize) -> Result<()> {
        let fleet = self.fleet(profile).await?;
        while fleet.clients.read().len() < replicas {
            let client = ApiClient::spawn(profile.spawn.clone()).await?;
            fleet.clients.write().push(client);
        }
        Ok(())
    }

    /// Leases a worker using round-robin selection.
    pub async fn lease(&self, profile: &ApiProfile) -> Result<ApiClient> {
        self.prewarm(profile, 1).await?;
        let fleet = self.fleet(profile).await?;
        let clients = fleet.clients.read();
        let index = fleet.next.fetch_add(1, Ordering::Relaxed) % clients.len();
        Ok(clients[index].clone())
    }

    /// Returns a cached snapshot or creates one lazily.
    pub async fn cached_snapshot(
        &self,
        profile: &ApiProfile,
        key: impl Into<CompactString>,
        params: UpdateSnapshotParams,
    ) -> Result<Arc<ManagedSnapshot>> {
        let key = key.into();
        if let Some(snapshot) = self.snapshots.read().get(key.as_str()) {
            return Ok(snapshot.clone());
        }
        let client = self.lease(profile).await?;
        let snapshot: Arc<ManagedSnapshot> = Arc::new(client.update_snapshot(params).await?);
        self.snapshots.write().insert(key, snapshot.clone());
        Ok(snapshot)
    }

    /// Invalidates a cached snapshot by key.
    pub fn invalidate_snapshot(&self, key: &str) {
        self.snapshots.write().remove(key);
    }

    /// Memoizes the result of an async task for an optional TTL.
    pub async fn cached<T, F, Fut>(
        &self,
        profile: &ApiProfile,
        key: impl Into<CompactString>,
        ttl: Option<Duration>,
        task: F,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce(ApiClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let key = key.into();
        if let Some(value) = self.cached.read().get(key.as_str())
            && value
                .expires_at
                .map(|deadline| deadline > Instant::now())
                .unwrap_or(true)
        {
            return Ok(serde_json::from_slice(&value.bytes)?);
        }
        let value = task(self.lease(profile).await?).await?;
        self.cached.write().insert(
            key,
            CachedValue {
                expires_at: ttl.map(|ttl| Instant::now() + ttl),
                bytes: serde_json::to_vec(&value)?.into_iter().collect(),
            },
        );
        Ok(value)
    }

    /// Executes the same task across a batch using multiple workers.
    pub async fn execute_all<T, F, Fut, I, R>(
        &self,
        profile: &ApiProfile,
        replicas: usize,
        inputs: I,
        task: F,
    ) -> Result<SmallVec<[R; 8]>>
    where
        I: IntoIterator<Item = T>,
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(ApiClient, T) -> Fut + Send + Sync + Copy + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
    {
        self.prewarm(profile, replicas).await?;
        let inputs = inputs.into_iter().collect::<SmallVec<[T; 8]>>();
        let work_count = inputs.len();
        let (work_tx, work_rx) = mpsc::channel::<(usize, T)>();
        let work_rx = Arc::new(std::sync::Mutex::new(work_rx));
        let (result_tx, result_rx) = mpsc::channel::<Result<(usize, R)>>();
        thread::scope(|scope| {
            for _ in 0..replicas.max(1) {
                let profile = profile.clone();
                let work_rx = work_rx.clone();
                let result_tx = result_tx.clone();
                scope.spawn(move || {
                    loop {
                        let job = work_rx.lock().unwrap().recv();
                        let Ok((index, input)) = job else {
                            break;
                        };
                        let output = block_on(async {
                            let client = self.lease(&profile).await?;
                            Ok::<_, crate::TsgoError>((index, task(client, input).await?))
                        });
                        if result_tx.send(output).is_err() {
                            break;
                        }
                    }
                });
            }
            drop(result_tx);
            for (index, input) in inputs.into_iter().enumerate() {
                work_tx
                    .send((index, input))
                    .map_err(|_| crate::TsgoError::Closed("orchestrator work"))?;
            }
            drop(work_tx);
            Ok::<_, crate::TsgoError>(())
        })?;
        let mut values = SmallVec::<[(usize, R); 8]>::new();
        for _ in 0..work_count {
            values.push(
                result_rx
                    .recv()
                    .map_err(|_| crate::TsgoError::Closed("orchestrator result"))??,
            );
        }
        values.sort_by_key(|(index, _)| *index);
        Ok(values.into_iter().map(|(_, value)| value).collect())
    }

    async fn fleet(&self, profile: &ApiProfile) -> Result<Arc<ClientFleet>> {
        if let Some(fleet) = self.fleets.read().get(profile.id.as_str()) {
            return Ok(fleet.clone());
        }
        let fleet = Arc::new(ClientFleet {
            next: AtomicUsize::new(0),
            clients: RwLock::new(SmallVec::new()),
        });
        self.fleets
            .write()
            .insert(profile.id.clone(), fleet.clone());
        Ok(fleet)
    }
}

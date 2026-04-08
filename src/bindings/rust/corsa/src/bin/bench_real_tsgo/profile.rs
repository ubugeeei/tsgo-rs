use std::{collections::BTreeMap, sync::Mutex, time::Duration};

use corsa::{
    api::{ApiProfileEvent, ApiProfiler},
    fast::{CompactString, SmallVec},
};

use crate::stats::Stats;

#[derive(Clone, Debug)]
pub struct ScenarioProfileRow {
    pub method: CompactString,
    pub phase: CompactString,
    pub stats: Stats,
}

#[derive(Default)]
pub struct BenchProfiler {
    events: Mutex<SmallVec<[ApiProfileEvent; 32]>>,
}

impl BenchProfiler {
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    pub fn drain_iteration_totals(&self) -> BTreeMap<(CompactString, CompactString), Duration> {
        let mut events = self.events.lock().unwrap();
        let drained = std::mem::take(&mut *events);
        let mut totals = BTreeMap::<(CompactString, CompactString), Duration>::new();
        for event in drained {
            let key = (event.method, CompactString::from(event.phase.as_str()));
            totals
                .entry(key)
                .and_modify(|duration| *duration += event.duration)
                .or_insert(event.duration);
        }
        totals
    }
}

impl ApiProfiler for BenchProfiler {
    fn on_profile(&self, event: &ApiProfileEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

pub fn summarize(
    samples: BTreeMap<(CompactString, CompactString), SmallVec<[Duration; 32]>>,
) -> SmallVec<[ScenarioProfileRow; 32]> {
    let mut rows = SmallVec::<[ScenarioProfileRow; 32]>::new();
    for ((method, phase), durations) in samples {
        rows.push(ScenarioProfileRow {
            method,
            phase,
            stats: Stats::from_samples(durations),
        });
    }
    rows
}

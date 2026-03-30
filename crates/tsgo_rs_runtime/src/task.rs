use crate::block_on;
use std::{future::Future, thread};

/// Join handle returned by [`spawn`].
pub struct JoinHandle<T> {
    inner: thread::JoinHandle<T>,
}

/// Spawns a future onto a dedicated worker thread.
///
/// # Examples
///
/// ```
/// use tsgo_rs_runtime::spawn;
///
/// let handle = spawn(async { String::from("tsgo") });
/// assert_eq!(handle.join().unwrap(), "tsgo");
/// ```
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandle {
        inner: thread::spawn(move || block_on(future)),
    }
}

impl<T> JoinHandle<T> {
    /// Waits for the worker thread to finish and returns its output.
    pub fn join(self) -> thread::Result<T> {
        self.inner.join()
    }
}

#[cfg(test)]
mod tests {
    use super::spawn;

    #[test]
    fn spawn_runs_future_on_worker_thread() {
        let handle = spawn(async { 5_u32 + 7 });
        assert_eq!(handle.join().unwrap(), 12);
    }
}

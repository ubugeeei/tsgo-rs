use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Condvar, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

/// Runs a future to completion on the current thread.
///
/// # Examples
///
/// ```
/// use tsgo_rs_runtime::block_on;
///
/// let value = block_on(async { 6 * 7 });
/// assert_eq!(value, 42);
/// ```
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    let parker = Arc::new(Parker::default());
    let waker = unsafe { Waker::from_raw(raw_waker(Arc::into_raw(parker.clone()) as *const ())) };
    let mut future = Pin::from(Box::new(future));
    let mut context = Context::from_waker(&waker);
    loop {
        parker.clear();
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
        parker.park();
    }
}

#[derive(Default)]
struct Parker {
    notified: Mutex<bool>,
    ready: Condvar,
}

impl Parker {
    fn clear(&self) {
        *self.notified.lock().unwrap() = false;
    }

    fn wake(&self) {
        let mut notified = self.notified.lock().unwrap();
        *notified = true;
        self.ready.notify_one();
    }

    fn park(&self) {
        let mut notified = self.notified.lock().unwrap();
        while !*notified {
            notified = self.ready.wait(notified).unwrap();
        }
    }
}

unsafe fn raw_waker(pointer: *const ()) -> RawWaker {
    RawWaker::new(pointer, &VTABLE)
}

unsafe fn clone_waker(pointer: *const ()) -> RawWaker {
    unsafe {
        std::sync::Arc::<Parker>::increment_strong_count(pointer.cast::<Parker>());
        raw_waker(pointer)
    }
}

unsafe fn wake_waker(pointer: *const ()) {
    let parker = unsafe { Arc::from_raw(pointer.cast::<Parker>()) };
    parker.wake();
}

unsafe fn wake_by_ref_waker(pointer: *const ()) {
    let parker = unsafe { Arc::from_raw(pointer.cast::<Parker>()) };
    parker.wake();
    let _ = Arc::into_raw(parker);
}

unsafe fn drop_waker(pointer: *const ()) {
    drop(unsafe { Arc::from_raw(pointer.cast::<Parker>()) });
}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_waker, wake_waker, wake_by_ref_waker, drop_waker);

#[cfg(test)]
mod tests {
    use super::block_on;

    #[test]
    fn block_on_runs_ready_future() {
        let value = block_on(async { 42 });
        assert_eq!(value, 42);
    }
}

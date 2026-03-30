use smallvec::SmallVec;
use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

/// Broadcast sender that clones each message for all active subscribers.
#[derive(Clone)]
pub struct Sender<T> {
    inner: Arc<Mutex<SmallVec<[mpsc::Sender<T>; 4]>>>,
}

/// Receiving side of a broadcast channel.
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

/// Creates a broadcast channel and returns the first receiver.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use tsgo_rs_runtime::broadcast;
///
/// let (sender, first) = broadcast();
/// let second = sender.subscribe();
///
/// assert_eq!(sender.send(7_u32), 2);
/// assert_eq!(first.recv_timeout(Duration::from_millis(50)).unwrap(), 7);
/// assert_eq!(second.recv_timeout(Duration::from_millis(50)).unwrap(), 7);
/// ```
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Mutex::new(SmallVec::<[mpsc::Sender<T>; 4]>::new()));
    let sender = Sender { inner };
    let receiver = sender.subscribe();
    (sender, receiver)
}

impl<T> Sender<T> {
    /// Creates a new receiver subscribed to future messages.
    pub fn subscribe(&self) -> Receiver<T> {
        let (tx, rx) = mpsc::channel();
        self.inner.lock().unwrap().push(tx);
        Receiver { inner: rx }
    }
}

impl<T> Sender<T>
where
    T: Clone,
{
    /// Sends a value to all active subscribers and returns the delivery count.
    pub fn send(&self, value: T) -> usize {
        let mut subscribers = self.inner.lock().unwrap();
        let mut delivered = 0;
        subscribers.retain(|subscriber| match subscriber.send(value.clone()) {
            Ok(()) => {
                delivered += 1;
                true
            }
            Err(_) => false,
        });
        delivered
    }
}

impl<T> Receiver<T> {
    /// Blocks until the next value arrives.
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.inner.recv()
    }

    /// Blocks until the next value arrives or the timeout expires.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, mpsc::RecvTimeoutError> {
        self.inner.recv_timeout(timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::channel;
    use std::time::Duration;

    #[test]
    fn broadcast_delivers_to_multiple_receivers() {
        let (sender, first) = channel::<u32>();
        let second = sender.subscribe();
        assert_eq!(sender.send(7_u32), 2);
        assert_eq!(first.recv_timeout(Duration::from_millis(50)).unwrap(), 7);
        assert_eq!(second.recv_timeout(Duration::from_millis(50)).unwrap(), 7);
    }

    #[test]
    fn send_prunes_dropped_receivers() {
        let (sender, first) = channel::<u32>();
        let second = sender.subscribe();
        drop(second);
        assert_eq!(sender.send(7_u32), 1);
        assert_eq!(first.recv_timeout(Duration::from_millis(50)).unwrap(), 7);
        assert_eq!(sender.send(8_u32), 1);
    }

    #[test]
    fn recv_reports_disconnect_when_all_senders_are_gone() {
        let (sender, receiver) = channel::<u32>();
        drop(sender);
        assert!(receiver.recv().is_err());
    }
}

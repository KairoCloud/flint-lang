use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub struct Channel<T> {
    inner: Arc<ChannelInner<T>>,
}

struct ChannelInner<T> {
    queue: Mutex<VecDeque<T>>,
    send_wait: Condvar,
    recv_wait: Condvar,
    closed: Mutex<bool>,
    capacity: usize,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Channel::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Channel {
            inner: Arc::new(ChannelInner {
                queue: Mutex::new(VecDeque::new()),
                send_wait: Condvar::new(),
                recv_wait: Condvar::new(),
                closed: Mutex::new(false),
                capacity,
            }),
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut queue = self.inner.queue.lock().unwrap();
        
        if *self.inner.closed.lock().unwrap() {
            return Err(SendError(value));
        }

        if self.inner.capacity > 0 {
            while queue.len() >= self.inner.capacity {
                queue = self.inner.send_wait.wait(queue).unwrap();
                if *self.inner.closed.lock().unwrap() {
                    return Err(SendError(value));
                }
            }
        }

        queue.push_back(value);
        self.inner.recv_wait.notify_one();
        Ok(())
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        let mut queue = self.inner.queue.lock().unwrap();

        while queue.is_empty() {
            if *self.inner.closed.lock().unwrap() {
                return Err(RecvError);
            }
            queue = self.inner.recv_wait.wait(queue).unwrap();
        }

        if *self.inner.closed.lock().unwrap() && queue.is_empty() {
            return Err(RecvError);
        }

        let value = queue.pop_front().unwrap();
        self.inner.send_wait.notify_one();
        Ok(value)
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let mut queue = self.inner.queue.lock().unwrap();
        
        if *self.inner.closed.lock().unwrap() && queue.is_empty() {
            return Err(TryRecvError);
        }

        queue.pop_front()
            .map(Ok)
            .unwrap_or_else(|| TryRecvError)
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let mut queue = self.inner.queue.lock().unwrap();

        while queue.is_empty() {
            if *self.inner.closed.lock().unwrap() {
                return Err(RecvTimeoutError);
            }
            let result = self.inner.recv_wait.wait_timeout(queue, timeout);
            queue = result.unwrap().0;
            if queue.is_empty() {
                return Err(RecvTimeoutError);
            }
        }

        if *self.inner.closed.lock().unwrap() && queue.is_empty() {
            return Err(RecvTimeoutError);
        }

        Ok(queue.pop_front().unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.queue.lock().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.queue.lock().unwrap().len()
    }

    pub fn close(&self) {
        *self.inner.closed.lock().unwrap() = true;
        self.inner.send_wait.notify_all();
        self.inner.recv_wait.notify_all();
    }
}

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Channel<T> {
        Channel {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sending on a closed channel")
    }
}

#[derive(Debug)]
pub struct RecvError;

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "receiving on a closed channel")
    }
}

#[derive(Debug)]
pub enum TryRecvError {
    Empty,
    Closed,
}

impl std::fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "channel empty"),
            TryRecvError::Closed => write!(f, "channel closed"),
        }
    }
}

#[derive(Debug)]
pub enum RecvTimeoutError {
    Timeout,
    Closed,
}

impl std::fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvTimeoutError::Timeout => write!(f, "recv timed out"),
            RecvTimeoutError::Closed => write!(f, "channel closed"),
        }
    }
}

pub struct Sender<T> {
    channel: Channel<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.channel.send(value)
    }

    pub fn is_closed(&self) -> bool {
        *self.channel.inner.closed.lock().unwrap()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        Sender { channel: self.channel.clone() }
    }
}

pub struct Receiver<T> {
    channel: Channel<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        self.channel.recv()
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.channel.try_recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.channel.recv_timeout(timeout)
    }

    pub fn is_closed(&self) -> bool {
        *self.channel.inner.closed.lock().unwrap()
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Receiver<T> {
        Receiver { channel: self.channel.clone() }
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let ch = Channel::new();
    (Sender { channel: ch.clone() }, Receiver { channel: ch })
}

pub fn bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let ch = Channel::with_capacity(capacity);
    (Sender { channel: ch.clone() }, Receiver { channel: ch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_basic() {
        let (tx, rx) = channel::<i32>();
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn test_channel_multiple() {
        let (tx, rx) = channel::<i32>();
        for i in 0..10 {
            tx.send(i).unwrap();
        }
        for i in 0..10 {
            assert_eq!(rx.recv().unwrap(), i);
        }
    }

    #[test]
    fn test_bounded() {
        let (tx, rx) = bounded::<i32>(2);
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        // Should block on third send
        let _ = tx.try_send(3);
    }

    #[test]
    fn test_close() {
        let (tx, rx) = channel::<i32>();
        tx.send(1).unwrap();
        tx.close();
        assert_eq!(rx.recv().unwrap(), 1);
        assert!(rx.recv().is_err());
    }
}
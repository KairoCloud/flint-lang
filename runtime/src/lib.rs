use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

pub mod async_runtime;
pub mod channel;
pub mod task;
pub mod actor;

pub use async_runtime::AsyncRuntime;
pub use channel::{Channel, Sender, Receiver};
pub use task::{Task, TaskId, TaskStatus};
pub use actor::Actor;

pub fn spawn_thread<F>(f: F) -> JoinHandle<F::Output>
where
    F: FnOnce() + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandle(thread::spawn(f))
}

pub struct JoinHandle<T> {
    handle: Option<thread::JoinHandle<T>>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, ()> {
        self.handle.map(|h| h.join()).transpose().map_err(|_| ())
    }
}

pub fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn sleep(duration: Duration) {
    thread::sleep(duration)
}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
    guard: Option<std::sync::MutexGuard<'a, T>>,
}

impl<'a, T> MutexGuard<'a, T> {
    fn new(lock: &'a Mutex<T>) -> Option<Self> {
        lock.try_lock().ok().map(|g| MutexGuard { lock, guard: Some(g) })
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(guard) = self.guard.take() {
            drop(guard);
        }
    }
}

impl<'a, T> std::ops::Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.as_mut().unwrap()
    }
}

pub struct RwLock<T> {
    data: Arc<Mutex<T>>,
    readers: Arc<Mutex<usize>>,
    writers: Arc<Mutex<usize>>,
}

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        RwLock {
            data: Arc::new(Mutex::new(data)),
            readers: Arc::new(Mutex::new(0)),
            writers: Arc::new(Mutex::new(0)),
        }
    }

    pub fn read(&self) -> Option<RwLockReadGuard<T>> {
        let _writers = self.writers.lock().ok()?;
        let mut readers = self.readers.lock().ok()?;
        *readers += 1;
        let data = self.data.lock().ok()?;
        Some(RwLockReadGuard { lock: self, data: Some(data), readers })
    }

    pub fn write(&self) -> Option<RwLockWriteGuard<T>> {
        let mut writers = self.writers.lock().ok()?;
        *writers += 1;
        let data = self.data.lock().ok()?;
        Some(RwLockWriteGuard { lock: self, data: Some(data), writers })
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
    data: Option<std::sync::MutexGuard<'a, T>>,
    readers: std::sync::MutexGuard<'a, usize>,
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        if let Ok(mut readers) = self.lock.readers.lock() {
            *readers -= 1;
        }
        self.data.take();
    }
}

impl<'a, T> std::ops::Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data.as_ref().unwrap()
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
    data: Option<std::sync::MutexGuard<'a, T>>,
    writers: std::sync::MutexGuard<'a, usize>,
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        if let Ok(mut writers) = self.lock.writers.lock() {
            *writers -= 1;
        }
        self.data.take();
    }
}

impl<'a, T> std::ops::Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data.as_mut().unwrap()
    }
}

pub mod collections {
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::hash::Hash;

    pub struct HashSet<T> {
        inner: std::collections::HashSet<T>,
    }

    impl<T: Eq + Hash> HashSet<T> {
        pub fn new() -> Self {
            HashSet { inner: std::collections::HashSet::new() }
        }
        pub fn insert(&mut self, value: T) -> bool {
            self.inner.insert(value)
        }
        pub fn contains(&self, value: &T) -> bool {
            self.inner.contains(value)
        }
        pub fn remove(&mut self, value: &T) -> bool {
            self.inner.remove(value)
        }
        pub fn len(&self) -> usize {
            self.inner.len()
        }
        pub fn iter(&self) -> impl Iterator<Item=&T> {
            self.inner.iter()
        }
    }

    pub struct HashMap<K, V> {
        inner: std::collections::HashMap<K, V>,
    }

    impl<K: Eq + Hash, V> HashMap<K, V> {
        pub fn new() -> Self {
            HashMap { inner: std::collections::HashMap::new() }
        }
        pub fn insert(&mut self, key: K, value: V) -> Option<V> {
            self.inner.insert(key, value)
        }
        pub fn get(&self, key: &K) -> Option<&V> {
            self.inner.get(key)
        }
        pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.inner.get_mut(key)
        }
        pub fn remove(&mut self, key: &K) -> Option<V> {
            self.inner.remove(key)
        }
        pub fn contains_key(&self, key: &K) -> bool {
            self.inner.contains_key(key)
        }
        pub fn len(&self) -> usize {
            self.inner.len()
        }
        pub fn keys(&self) -> impl Iterator<Item=&K> {
            self.inner.keys()
        }
        pub fn values(&self) -> impl Iterator<Item=&V> {
            self.inner.values()
        }
    }

    pub struct VecDeque<T> {
        inner: std::collections::VecDeque<T>,
    }

    impl<T> VecDeque<T> {
        pub fn new() -> Self {
            VecDeque { inner: std::collections::VecDeque::new() }
        }
        pub fn push_back(&mut self, value: T) {
            self.inner.push_back(value)
        }
        pub fn push_front(&mut self, value: T) {
            self.inner.push_front(value)
        }
        pub fn pop_front(&mut self) -> Option<T> {
            self.inner.pop_front()
        }
        pub fn pop_back(&mut self) -> Option<T> {
            self.inner.pop_back()
        }
        pub fn front(&self) -> Option<&T> {
            self.inner.front()
        }
        pub fn back(&self) -> Option<&T> {
            self.inner.back()
        }
        pub fn len(&self) -> usize {
            self.inner.len()
        }
        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel() {
        let (tx, rx) = channel::<i32>();
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn test_spawn() {
        let handle = spawn_thread(|| {
            42
        });
        assert_eq!(handle.join().unwrap(), 42);
    }

    #[test]
    fn test_mutex() {
        let data = Arc::new(Mutex::new(0));
        {
            let mut guard = data.lock().unwrap();
            *guard = 42;
        }
        assert_eq!(*data.lock().unwrap(), 42);
    }

    #[test]
    fn test_rwlock() {
        let lock = RwLock::new(42);
        {
            let mut guard = lock.write().unwrap();
            *guard = 100;
        }
        {
            let guard = lock.read().unwrap();
            assert_eq!(*guard, 100);
        }
    }
}
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

use crate::channel::Channel;

pub struct AsyncRuntime {
    tasks: Arc<Mutex<HashMap<TaskId, TaskState>>>,
    runqueue: Arc<Mutex<VecDeque<TaskId>>>,
    worker_pool: Vec<thread::JoinHandle<()>>,
    event_loop: Option<thread::JoinHandle<()>>,
    shutdown: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct TaskId(u64);

impl TaskId {
    fn new(id: u64) -> Self {
        TaskId(id)
    }
}

enum TaskState {
    Ready(Pin<Box<dyn Future<Output = ()> + Send>>),
    Running,
    Completed,
    Blocked(TaskId),
}

pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

impl<T, E> Future for Result<T, E> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_ref().as_ref() {
            Ok(t) => Poll::Ready(t.clone()),
            Err(_) => Poll::Pending,
        }
    }
}

impl<F: Future + ?Sized> Future for Box<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        F::poll(self.as_mut().get_unchecked_mut(), cx)
    }
}

pub struct Sleep {
    deadline: Option<Duration>,
    waker: Option<Waker>,
}

impl Sleep {
    fn new(duration: Duration) -> Self {
        Sleep {
            deadline: Some(duration),
            waker: None,
        }
    }
}

impl Future for Sleep {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match self.deadline {
            Some(d) => {
                if d <= Duration::from_secs(0) {
                    Poll::Ready(())
                } else {
                    self.waker = Some(cx.waker().clone());
                    self.deadline = Some(d - Duration::from_millis(1));
                    Poll::Pending
                }
            }
            None => Poll::Ready(()),
        }
    }
}

pub struct Timeout<T> {
    inner: Pin<Box<T>>,
    deadline: Option<Duration>,
    waker: Option<Waker>,
}

impl<T> Timeout<T> {
    fn new(inner: Pin<Box<T>>, duration: Duration) -> Self {
        Timeout {
            inner,
            deadline: Some(duration),
            waker: None,
        }
    }
}

impl<T> Future for Timeout<T> {
    type Output = T::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.deadline.map(|d| d <= Duration::from_secs(0)).unwrap_or(false) {
            return Poll::Ready(panic!("timeout"));
        }
        T::poll(self.inner.as_mut(), cx)
    }
}

impl AsyncRuntime {
    pub fn new() -> AsyncRuntime {
        let num_threads = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        let tasks = Arc::new(Mutex::new(HashMap::new()));
        let runqueue = Arc::new(Mutex::new(VecDeque::new()));
        let shutdown = Arc::new(Mutex::new(false));

        let worker_tasks = tasks.clone();
        let worker_runqueue = runqueue.clone();
        let worker_shutdown = shutdown.clone();

        let workers: Vec<_> = (0..num_threads)
            .map(|_| {
                thread::spawn(move || {
                    loop {
                        let task_id = {
                            let mut q = worker_runqueue.lock().unwrap();
                            q.pop_front()
                        };

                        if let Some(id) = task_id {
                            let should_run = {
                                let mut t = worker_tasks.lock().unwrap();
                                if let Some(state) = t.get_mut(&id) {
                                    if let TaskState::Ready(_) = state {
                                        *state = TaskState::Running;
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            };

                            if should_run {
                                let _ = worker_tasks.lock().unwrap().remove(&id);
                            }
                        } else {
                            thread::sleep(Duration::from_millis(1));
                        }

                        if *worker_shutdown.lock().unwrap() {
                            break;
                        }
                    }
                })
            })
            .collect();

        AsyncRuntime {
            tasks,
            runqueue,
            worker_pool: workers,
            event_loop: None,
            shutdown,
        }
    }

    pub fn spawn<F>(&self, future: F) -> TaskId
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = TaskId::new(self.tasks.lock().unwrap().len() as u64);
        self.tasks.lock().unwrap().insert(
            id.0,
            TaskState::Ready(Box::pin(future)),
        );
        self.runqueue.lock().unwrap().push_back(id.0);
        id
    }

    pub fn spawn_blocking<F, T>(&self, f: F) -> TaskId
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = Channel::new();
        let result: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
        let result_clone = result.clone();

        thread::spawn(move || {
            let output = f();
            *result_clone.lock().unwrap() = Some(output);
        });

        let id = TaskId::new(self.tasks.lock().unwrap().len() as u64);
        self.tasks.lock().unwrap().insert(
            id.0,
            TaskState::Ready(Box::pin(async move {
                let _ = rx.recv();
            })),
        );
        self.runqueue.lock().unwrap().push_back(id.0);
        id
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut future = Box::pin(future);

        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }
    }

    pub fn shutdown(&self) {
        *self.shutdown.lock().unwrap() = true;
        for worker in self.worker_pool.drain(..) {
            let _ = worker.join();
        }
    }

    pub fn num_workers(&self) -> usize {
        self.worker_pool.len()
    }
}

fn noop_waker() -> Waker {
    struct NoopWaker;
    impl Waker for NoopWaker {
        fn wake(self: Arc<Self>) {}
        fn wake_by_ref(&self) {}
    }
    unsafe { Waker::from_arc(Arc::new(NoopWaker)) }
}

pub async fn sleep(duration: Duration) {
    Sleep::new(duration).await
}

pub async fn timeout<T: Future>(duration: Duration, future: T) -> T::Output {
    Timeout::new(Box::pin(future), duration).await
}

pub async fn yield_now() {
    Sleep::new(Duration::from_secs(0)).await
}

pub struct JoinSet {
    runtime: AsyncRuntime,
    tasks: Vec<TaskId>,
}

impl JoinSet {
    pub fn new() -> JoinSet {
        JoinSet {
            runtime: AsyncRuntime::new(),
            tasks: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.runtime.spawn(future);
        self.tasks.push(id);
    }

    pub async fn join_all(&mut self) {
        for task in self.tasks.drain(..) {
            // Wait for each task
        }
    }
}

pub struct select {
    pub left: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
    pub right: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl select {
    pub fn new() -> Self {
        select {
            left: None,
            right: None,
        }
    }

    pub fn left<F: Future + Send + 'static>(mut self, future: F) -> Self {
        self.left = Some(Box::pin(future));
        self
    }

    pub fn right<F: Future + Send + 'static>(mut self, future: F) -> Self {
        self.right = Some(Box::pin(future));
        self
    }
}

impl Future for select {
    type Output = bool;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<bool> {
        let this = self.get_mut();
        if let Some(ref mut left) = this.left {
            return Poll::Ready(true);
        }
        if let Some(ref mut right) = this.right {
            return Poll::Ready(false);
        }
        Poll::Pending
    }
}

pub async fn race<F1, F2>(f1: F1, f2: F2) -> F1::Output
where
    F1: Future,
    F2: Future<Output = F1::Output>,
{
    select::new().left(f1).right(f2).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = AsyncRuntime::new();
        assert!(runtime.num_workers() >= 1);
        runtime.shutdown();
    }

    #[test]
    fn test_spawn_task() {
        let runtime = AsyncRuntime::new();
        let id = runtime.spawn(async { 42 });
        runtime.shutdown();
    }
}
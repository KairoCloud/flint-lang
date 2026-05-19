use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

static TASK_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> TaskId {
        TaskId(TASK_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for TaskId {
    fn default() -> Self {
        TaskId::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Completed,
    Failed(String),
    Cancelled,
    Blocked,
}

pub struct Task {
    id: TaskId,
    name: String,
    status: Arc<Mutex<TaskStatus>>,
    result: Arc<Mutex<Option<Result<(), String>>>>,
    priority: u8,
    created_at: Instant,
}

impl Task {
    pub fn new(name: String) -> Self {
        Task {
            id: TaskId::new(),
            name,
            status: Arc::new(Mutex::new(TaskStatus::Ready)),
            result: Arc::new(Mutex::new(None)),
            priority: 0,
            created_at: Instant::now(),
        }
    }

    pub fn with_priority(name: String, priority: u8) -> Self {
        Task {
            id: TaskId::new(),
            name,
            status: Arc::new(Mutex::new(TaskStatus::Ready)),
            result: Arc::new(Mutex::new(None)),
            priority,
            created_at: Instant::now(),
        }
    }

    pub fn id(&self) -> TaskId {
        self.id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> TaskStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_status(&self, status: TaskStatus) {
        *self.status.lock().unwrap() = status;
    }

    pub fn set_result(&self, result: Result<(), String>) {
        *self.result.lock().unwrap() = Some(result);
    }

    pub fn result(&self) -> Option<Result<(), String>> {
        self.result.lock().unwrap().clone()
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn cancel(&self) {
        *self.status.lock().unwrap() = TaskStatus::Cancelled;
    }

    pub fn is_cancelled(&self) -> bool {
        *self.status.lock().unwrap() == TaskStatus::Cancelled
    }

    pub fn is_completed(&self) -> bool {
        matches!(*self.status.lock().unwrap(), TaskStatus::Completed)
    }

    pub fn is_running(&self) -> bool {
        matches!(*self.status.lock().unwrap(), TaskStatus::Running)
    }
}

pub struct TaskRegistry {
    tasks: Arc<Mutex<Vec<Arc<Task>>>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        TaskRegistry {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&self, task: Arc<Task>) {
        self.tasks.lock().unwrap().push(task);
    }

    pub fn unregister(&self, id: &TaskId) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.retain(|t| t.id() != *id);
    }

    pub fn get(&self, id: &TaskId) -> Option<Arc<Task>> {
        self.tasks.lock().unwrap()
            .iter()
            .find(|t| t.id() == *id)
            .cloned()
    }

    pub fn all(&self) -> Vec<Arc<Task>> {
        self.tasks.lock().unwrap().clone()
    }

    pub fn running(&self) -> Vec<Arc<Task>> {
        self.tasks.lock().unwrap()
            .iter()
            .filter(|t| t.is_running())
            .cloned()
            .collect()
    }

    pub fn completed(&self) -> Vec<Arc<Task>> {
        self.tasks.lock().unwrap()
            .iter()
            .filter(|t| t.is_completed())
            .cloned()
            .collect()
    }

    pub fn clear_completed(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.retain(|t| !t.is_completed());
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CancellationToken {
    cancelled: Arc<Mutex<bool>>,
}

impl CancellationToken {
    pub fn new() -> Self {
        CancellationToken {
            cancelled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn cancel(&self) {
        *self.cancelled.lock().unwrap() = true;
    }

    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().unwrap()
    }

    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task cancelled")
    }
}

pub struct TaskBuilder {
    name: String,
    priority: u8,
    spawn: bool,
}

impl TaskBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        TaskBuilder {
            name: name.into(),
            priority: 0,
            spawn: false,
        }
    }

    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    pub fn spawn<F>(self, f: F) -> Arc<Task>
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        let task = Arc::new(Task::with_priority(self.name, self.priority));
        let status = task.status.clone();
        let result = task.result.clone();

        thread::spawn(move || {
            *status.lock().unwrap() = TaskStatus::Running;
            let res = f();
            *status.lock().unwrap() = TaskStatus::Completed;
            *result.lock().unwrap() = Some(res);
        });

        task
    }

    pub fn build(self) -> Arc<Task> {
        Arc::new(Task::with_priority(self.name, self.priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("test".to_string());
        assert_eq!(task.name(), "test");
        assert_eq!(task.status(), TaskStatus::Ready);
    }

    #[test]
    fn test_task_status() {
        let task = Task::new("test".to_string());
        task.set_status(TaskStatus::Running);
        assert_eq!(task.status(), TaskStatus::Running);
        task.set_status(TaskStatus::Completed);
        assert!(task.is_completed());
    }

    #[test]
    fn test_cancellation() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_task_registry() {
        let registry = TaskRegistry::new();
        let task = Arc::new(Task::new("test".to_string()));
        registry.register(task.clone());
        assert_eq!(registry.all().len(), 1);
        assert_eq!(registry.get(&task.id()).unwrap().name(), "test");
    }
}
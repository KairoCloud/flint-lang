use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct JobQueue {
    jobs: Arc<Mutex<VecDeque<Job>>>,
    running: Arc<Mutex<VecDeque<Job>>>,
    max_workers: usize,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub payload: String,
    pub priority: i32,
    pub status: JobStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

impl Job {
    pub fn new(payload: &str) -> Self {
        Job {
            id: rand_id(),
            payload: payload.to_string(),
            priority: 0,
            status: JobStatus::Queued,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl JobQueue {
    pub fn new(max_workers: usize) -> Self {
        JobQueue {
            jobs: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(Mutex::new(VecDeque::new())),
            max_workers,
        }
    }

    pub fn enqueue(&self, job: Job) {
        self.jobs.lock().unwrap().push_back(job);
    }

    pub fn dequeue(&self) -> Option<Job> {
        self.jobs.lock().unwrap().pop_front()
    }

    pub fn start_workers(&self, handler: impl Fn(Job) + Send + Sync + 'static) {
        let jobs = self.jobs.clone();
        let running = self.running.clone();
        
        for _ in 0..self.max_workers {
            let jobs = jobs.clone();
            let running = running.clone();
            
            thread::spawn(move || {
                loop {
                    let job = jobs.lock().unwrap().pop_front();
                    if let Some(mut j) = job {
                        j.status = JobStatus::Running;
                        running.lock().unwrap().push_back(j.clone());
                        handler(j);
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            });
        }
    }

    pub fn size(&self) -> usize {
        self.jobs.lock().unwrap().len()
    }

    pub fn running_count(&self) -> usize {
        self.running.lock().unwrap().len()
    }
}

fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    format!("job-{:x}", nanos)
}

pub struct JobBuilder {
    payload: String,
    priority: i32,
}

impl JobBuilder {
    pub fn new(payload: &str) -> Self {
        JobBuilder {
            payload: payload.to_string(),
            priority: 0,
        }
    }

    pub fn priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }

    pub fn build(self) -> Job {
        Job::new(&self.payload).with_priority(self.priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_queue() {
        let queue = JobQueue::new(2);
        queue.enqueue(Job::new("task1"));
        queue.enqueue(Job::new("task2"));
        assert_eq!(queue.size(), 2);
    }

    #[test]
    fn test_job_builder() {
        let job = JobBuilder::new("process")
            .priority(10)
            .build();
        assert_eq!(job.priority, 10);
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::channel::{bounded, Channel, Receiver, Sender};
use crate::task::TaskId;

pub trait Actor: Send + 'static {
    type Message: Send;

    fn receive(&mut self, msg: Self::Message);

    fn handle_exception(&mut self, error: Box<dyn std::error::Error + Send>) {
        eprintln!("Actor error: {:?}", error);
    }
}

pub struct ActorRef<M: Send> {
    sender: Sender<ActorMessage<M>>,
    pub id: ActorId,
}

#[derive(Clone)]
pub struct ActorId(u64);

impl ActorId {
    static mut COUNTER: u64 = 0;
    pub fn new() -> Self {
        unsafe {
            ActorId(COUNTER)
        }
    }
}

enum ActorMessage<M: Send> {
    Handle(M),
    Stop,
}

pub struct ActorSystem {
    actors: Arc<Mutex<HashMap<ActorId, ActorHandle>>>,
}

struct ActorHandle {
    name: String,
    mailbox: Sender<ActorMessage<M>>,
    #[allow(dead_code)]
    thread: thread::JoinHandle<()>,
}

impl ActorSystem {
    pub fn new() -> Self {
        ActorSystem {
            actors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn spawn<A>(&self, name: impl Into<String>, actor: A) -> ActorRef<A::Message>
    where
        A: Actor,
    {
        let (tx, rx) = bounded(100);
        let id = ActorId::new();
        let actor_name = name.into();

        let actors = self.actors.clone();
        let actor_id = id.clone();

        let thread = thread::spawn(move || {
            Self::run_actor(actor_name, actor, rx);
            actors.lock().unwrap().remove(&actor_id);
        });

        self.actors.lock().unwrap().insert(
            id.clone(),
            ActorHandle {
                name: actor_name,
                mailbox: tx,
                thread,
            },
        );

        ActorRef {
            sender: tx,
            id,
        }
    }

    fn run_actor<M, A>(name: String, mut actor: A, rx: Receiver<ActorMessage<M>>)
    where
        M: Send,
        A: Actor<Message = M>,
    {
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(ActorMessage::Handle(msg)) => {
                    actor.receive(msg);
                }
                Ok(ActorMessage::Stop) => break,
                Err(_) => {
                    // Timeout, continue
                }
            }
        }
    }

    pub fn stop(&self, ref_: ActorRef<()>) {
        let _ = ref_.sender.send(ActorMessage::Stop);
    }

    pub fn shutdown(&self) {
        for (_, handle) in self.actors.lock().unwrap().drain() {
            let _ = handle.mailbox.send(ActorMessage::Stop);
            let _ = handle.thread.join();
        }
    }

    pub fn actor_count(&self) -> usize {
        self.actors.lock().unwrap().len()
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Send> ActorRef<M> {
    pub fn send(&self, msg: M) {
        let _ = self.sender.send(ActorMessage::Handle(msg));
    }

    pub fn send_timeout(&self, msg: M, timeout: Duration) -> Result<(), ()> {
        self.sender.send(msg).map_err(|_| ())
    }
}

impl<M: Send> Clone for ActorRef<M> {
    fn clone(&self) -> ActorRef<M> {
        ActorRef {
            sender: self.sender.clone(),
            id: self.id.clone(),
        }
    }
}

pub struct ActorBuilder<A: Actor> {
    name: String,
    actor: A,
    mailbox_size: usize,
}

impl<A: Actor> ActorBuilder<A> {
    pub fn new(actor: A) -> Self {
        ActorBuilder {
            name: "actor".to_string(),
            actor,
            mailbox_size: 100,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn mailbox_size(mut self, size: usize) -> Self {
        self.mailbox_size = size;
        self
    }

    pub fn spawn(self, system: &ActorSystem) -> ActorRef<A::Message> {
        system.spawn(self.name, self.actor)
    }
}

pub struct ActorRegistry {
    actors: Arc<Mutex<HashMap<String, ActorRef<()>>>>,
}

impl ActorRegistry {
    pub fn new() -> Self {
        ActorRegistry {
            actors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, name: String, ref_: ActorRef<()>) {
        self.actors.lock().unwrap().insert(name, ref_);
    }

    pub fn lookup(&self, name: &str) -> Option<ActorRef<()>> {
        self.actors.lock().unwrap().get(name).cloned()
    }

    pub fn unregister(&self, name: &str) {
        self.actors.lock().unwrap().remove(name);
    }
}

impl Default for ActorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Ask<T> {
    request: T,
    response: Channel<Box<dyn std::any::Any + Send>>,
}

impl<T: Send> Ask<T> {
    pub fn new(request: T) -> Self {
        Ask {
            request,
            response: Channel::new(),
        }
    }
}

pub fn ask<M: Send, R: Send>(ref_: &ActorRef<M>, request: M) -> Receiver<R> {
    let (tx, rx) = Channel::new();
    ref_.send(request);
    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter {
        count: i32,
    }

    impl Actor for Counter {
        type Message = i32;

        fn receive(&mut self, msg: i32) {
            self.count += msg;
        }
    }

    #[test]
    fn test_actor_spawn() {
        let system = ActorSystem::new();
        let ref_ = system.spawn("counter", Counter { count: 0 });
        ref_.send(5);
        ref_.send(3);
        system.shutdown();
    }

    #[test]
    fn test_actor_ref_clone() {
        let system = ActorSystem::new();
        let ref1 = system.spawn("test", Counter { count: 0 });
        let ref2 = ref1.clone();
        ref1.send(1);
        ref2.send(2);
        drop(ref1);
        drop(ref2);
        system.shutdown();
    }
}
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub struct State<T: Clone> {
    value: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
}

impl<T: Clone> State<T> {
    pub fn new(value: T) -> Self {
        State {
            value: Arc::new(RwLock::new(value)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value.clone();
        self.notify();
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut v = self.value.write().unwrap();
        f(&mut v);
        let new_value = v.clone();
        drop(v);
        self.notify();
    }

    pub fn subscribe(&self, callback: Box<dyn Fn(&T) + Send + Sync>) {
        self.subscribers.write().unwrap().push(callback);
    }

    fn notify(&self) {
        let value = self.get();
        for sub in self.subscribers.read().unwrap().iter() {
            sub(&value);
        }
    }
}

impl<T: Clone> Clone for State<T> {
    fn clone(&self) -> Self {
        State {
            value: self.value.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

pub struct StateManager {
    states: HashMap<String, Box<dyn std::any::Any>>,
}

impl StateManager {
    pub fn new() -> Self {
        StateManager {
            states: HashMap::new(),
        }
    }

    pub fn register<T: Clone + 'static>(&mut self, name: &str, state: State<T>) {
        self.states.insert(name.to_string(), Box::new(state));
    }

    pub fn get<T: Clone + 'static>(&self, name: &str) -> Option<State<T>> {
        self.states.get(name).and_then(|s| s.downcast_ref::<State<T>>()).cloned()
    }

    pub fn remove(&mut self, name: &str) {
        self.states.remove(name);
    }

    pub fn clear(&mut self) {
        self.states.clear();
    }
}

impl Default for StateManager {
    fn default() -> Self { Self::new() }
}

pub fn state<T: Clone>(initial: T) -> State<T> {
    State::new(initial)
}

pub fn derived<T: Clone, S: Clone>(source: State<S>, f: impl Fn(&S) -> T) -> State<T> {
    let derived = State::new(f(&source.get()));
    source.subscribe(Box::new(move |v| {
        derived.set(f(v));
    }));
    derived
}

pub fn effect(f: impl FnOnce() + Send + 'static) {
    // Run effect after render
    std::thread::spawn(f);
}

pub fn effect_on<T: Clone>(state: State<T>, f: impl Fn(&T) + Send + 'static) {
    state.subscribe(Box::new(f));
}

#[macro_export]
macro_rules! state {
    ($value:expr) => {
        ::flint_ui::state($value)
    };
    (mut $name:ident = $value:expr) => {
        let $name = ::flint_ui::state($value);
    };
}

#[macro_export]
macro_rules! derived {
    ($source:expr => $expr:expr) => {
        ::flint_ui::derived($source, |v| $expr)
    };
}

#[macro_export]
macro_rules! effect {
    ($($code:tt)*) => {
        ::flint_ui::effect(|| { $($code)* });
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state() {
        let s = state(42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_state_set() {
        let s = state(0);
        s.set(10);
        assert_eq!(s.get(), 10);
    }

    #[test]
    fn test_derived() {
        let source = state(5);
        let doubled = derived(source, |v| v * 2);
        assert_eq!(doubled.get(), 10);
    }
}
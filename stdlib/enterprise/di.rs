use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait Inject: 'static {
    type Output;
}

pub trait Provider<T>: Send + Sync {
    fn provide(&self) -> Box<dyn Fn() -> T + Send + Sync>;
}

pub enum Lifetime {
    Transient,
    Singleton,
    Scoped,
}

pub struct Container {
    providers: Arc<RwLock<HashMap<TypeId, Box<dyn AnyProvider>>>>,
    instances: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

trait AnyProvider: Send + Sync {
    fn clone_box(&self) -> Box<dyn AnyProvider>;
    fn provide_any(&self) -> Box<dyn Any + Send + Sync>;
}

impl<T: 'static> Clone for Box<dyn AnyProvider> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<F: Fn() -> T + Send + Sync + 'static, T: 'static> Provider<T> for F {
    fn provide(&self) -> Box<dyn Fn() -> T + Send + Sync> {
        Box::new(move || self())
    }
}

struct ConcreteProvider<T: 'static> {
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: 'static> AnyProvider for ConcreteProvider<T> {
    fn clone_box(&self) -> Box<dyn AnyProvider> {
        Box::new(ConcreteProvider { factory: self.factory.clone() })
    }

    fn provide_any(&self) -> Box<dyn Any + Send + Sync> {
        Box::new((self.factory)())
    }
}

impl Container {
    pub fn new() -> Self {
        Container {
            providers: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register<T: 'static, F: Fn() -> T + Send + Sync + 'static>(&self, factory: F) {
        let provider = ConcreteProvider { factory: Box::new(factory) };
        self.providers.write().unwrap().insert(TypeId::of::<T>(), provider);
    }

    pub fn register_instance<T: 'static>(&self, instance: T) {
        self.instances.write().unwrap().insert(TypeId::of::<T>(), Box::new(instance));
    }

    pub fn resolve<T: 'static>(&self) -> Result<T, String> {
        if let Some(instance) = self.instances.read().unwrap().get(&TypeId::of::<T>()) {
            return instance.downcast_ref::<T>().cloned()
                .ok_or_else(|| "type mismatch".to_string());
        }

        if let Some(provider) = self.providers.read().unwrap().get(&TypeId::of::<T>()) {
            let instance = provider.provide_any();
            return instance.downcast::<T>().map_err(|_| "type mismatch".to_string());
        }

        Err(format!("no provider registered for {:?}", TypeId::of::<T>()))
    }
}

impl Default for Container {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy)]
struct TypeId {
    name: &'static str,
}

impl TypeId {
    fn of<T: 'static>() -> Self {
        TypeId { name: std::any::type_name::<T>() }
    }
}

pub struct Injectable<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> Inject for Injectable<T> {
    type Output = T;
}

pub struct Singleton<T>(T);

pub struct Transient<T>(T);

#[macro_export]
macro_rules! inject {
    ($container:expr => $type:ty) => {
        $container.resolve::<$type>()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container() {
        let container = Container::new();
        container.register(|| 42);
        let result: i32 = container.resolve().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_instance() {
        let container = Container::new();
        container.register_instance("hello".to_string());
        let result: String = container.resolve().unwrap();
        assert_eq!(result, "hello");
    }
}
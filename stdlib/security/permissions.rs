use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    FileRead,
    FileWrite,
    FileExecute,
    NetworkOutbound,
    NetworkInbound,
    EnvRead,
    EnvWrite,
    ProcessSpawn,
    ThreadCreate,
    MemoryAllocate,
    Signal,
    SystemInfo,
}

impl Capability {
    pub fn name(&self) -> &str {
        match self {
            Capability::FileRead => "fs.read",
            Capability::FileWrite => "fs.write",
            Capability::FileExecute => "fs.execute",
            Capability::NetworkOutbound => "net.outbound",
            Capability::NetworkInbound => "net.inbound",
            Capability::EnvRead => "env.read",
            Capability::EnvWrite => "env.write",
            Capability::ProcessSpawn => "proc.spawn",
            Capability::ThreadCreate => "thread.create",
            Capability::MemoryAllocate => "mem.allocate",
            Capability::Signal => "signal",
            Capability::SystemInfo => "sys.info",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub required: bool,
}

impl Permission {
    pub fn new(name: &str, description: &str) -> Self {
        Permission {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    capabilities: HashMap<String, Capability>,
}

impl PermissionSet {
    pub fn new() -> Self {
        PermissionSet {
            capabilities: HashMap::new(),
        }
    }

    pub fn add(&mut self, cap: Capability) {
        self.capabilities.insert(cap.name().to_string(), cap);
    }

    pub fn add_all(&mut self, caps: Vec<Capability>) {
        for cap in caps {
            self.add(cap);
        }
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains_key(cap.name())
    }

    pub fn remove(&mut self, cap: &Capability) {
        self.capabilities.remove(cap.name());
    }

    pub fn clear(&mut self) {
        self.capabilities.clear();
    }

    pub fn merge(&mut self, other: &PermissionSet) {
        for (_, cap) in &other.capabilities {
            self.add(cap.clone());
        }
    }

    pub fn list(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }
}

pub struct PermissionChecker {
    required: PermissionSet,
    granted: Arc<RwLock<PermissionSet>>,
}

impl PermissionChecker {
    pub fn new(required: PermissionSet) -> Self {
        PermissionChecker {
            required,
            granted: Arc::new(RwLock::new(PermissionSet::new())),
        }
    }

    pub fn grant(&self, cap: Capability) {
        if let Ok(mut granted) = self.granted.write() {
            granted.add(cap);
        }
    }

    pub fn check(&self) -> Result<(), PermissionDenied> {
        let granted = self.granted.read().map_err(|_| PermissionDenied("lock error".to_string()))?;
        
        for cap in self.required.list() {
            if !granted.has(cap) {
                return Err(PermissionDenied(format!("missing capability: {}", cap.name())));
            }
        }
        Ok(())
    }

    pub fn check_or_die(&self) {
        self.check().unwrap();
    }
}

#[derive(Debug)]
pub struct PermissionDenied(String);

impl std::fmt::Display for PermissionDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "permission denied: {}", self.0)
    }
}

pub fn require(cap: Capability) {
    let perms = PermissionSet::new();
    let checker = PermissionChecker::new(perms);
    checker.check_or_die();
}

pub fn require_all(caps: Vec<Capability>) {
    let mut perms = PermissionSet::new();
    perms.add_all(caps);
    let checker = PermissionChecker::new(perms);
    checker.check_or_die();
}

#[macro_export]
macro_rules! require {
    ($cap:expr) => {
        ::flint_security::require($cap)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set() {
        let mut set = PermissionSet::new();
        set.add(Capability::FileRead);
        assert!(set.has(&Capability::FileRead));
        assert!(!set.has(&Capability::FileWrite));
    }

    #[test]
    fn test_permission_checker() {
        let mut required = PermissionSet::new();
        required.add(Capability::NetworkOutbound);
        
        let checker = PermissionChecker::new(required);
        checker.grant(Capability::NetworkOutbound);
        
        assert!(checker.check().is_ok());
    }
}
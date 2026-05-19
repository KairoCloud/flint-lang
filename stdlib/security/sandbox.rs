use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct Sandbox {
    config: SandboxConfig,
    permissions: Arc<Mutex<HashSet<String>>>,
    resources: Arc<Mutex<ResourceUsage>>,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub allow_fs: bool,
    pub allow_network: bool,
    pub allow_env: bool,
    pub allow_threads: bool,
    pub max_memory: usize,
    pub max_cpu_time: u64,
    pub max_file_size: usize,
    pub max_network_connections: usize,
    pub allowed_dirs: Vec<String>,
    pub allowed_hosts: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            allow_fs: false,
            allow_network: false,
            allow_env: false,
            allow_threads: false,
            max_memory: 100 * 1024 * 1024, // 100MB
            max_cpu_time: 60, // 60 seconds
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_network_connections: 0,
            allowed_dirs: Vec::new(),
            allowed_hosts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceLimit {
    pub memory: usize,
    pub cpu_time: u64,
    pub file_size: usize,
    pub network_connections: usize,
}

impl Default for ResourceLimit {
    fn default() -> Self {
        ResourceLimit {
            memory: 50 * 1024 * 1024,
            cpu_time: 30,
            file_size: 5 * 1024 * 1024,
            network_connections: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_used: usize,
    pub cpu_time_used: u64,
    pub files_created: usize,
    pub network_requests: usize,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Sandbox {
            config,
            permissions: Arc::new(Mutex::new(HashSet::new())),
            resources: Arc::new(Mutex::new(ResourceUsage::default())),
        }
    }

    pub fn allow_fs(&self, paths: Vec<String>) -> Result<(), String> {
        if !self.config.allow_fs {
            return Err("filesystem access not allowed".to_string());
        }
        
        let mut perms = self.permissions.lock().unwrap();
        for path in paths {
            perms.insert(format!("fs:{}", path));
        }
        Ok(())
    }

    pub fn allow_network(&self, hosts: Vec<String>) -> Result<(), String> {
        if !self.config.allow_network {
            return Err("network access not allowed".to_string());
        }
        
        let mut perms = self.permissions.lock().unwrap();
        for host in hosts {
            perms.insert(format!("net:{}", host));
        }
        Ok(())
    }

    pub fn allow_env(&self, vars: Vec<String>) -> Result<(), String> {
        if !self.config.allow_env {
            return Err("environment access not allowed".to_string());
        }
        
        let mut perms = self.permissions.lock().unwrap();
        for var in vars {
            perms.insert(format!("env:{}", var));
        }
        Ok(())
    }

    pub fn check_permission(&self, permission: &str) -> bool {
        self.permissions.lock().unwrap().contains(permission)
    }

    pub fn check_fs(&self, path: &str) -> bool {
        self.check_permission(&format!("fs:{}", path))
    }

    pub fn check_network(&self, host: &str) -> bool {
        self.check_permission(&format!("net:{}", host))
    }

    pub fn check_env(&self, var: &str) -> bool {
        self.check_permission(&format!("env:{}", var))
    }

    pub fn record_memory(&self, bytes: usize) -> Result<(), String> {
        let mut usage = self.resources.lock().unwrap();
        usage.memory_used += bytes;
        
        if usage.memory_used > self.config.max_memory {
            return Err("memory limit exceeded".to_string());
        }
        Ok(())
    }

    pub fn record_cpu_time(&self, seconds: u64) -> Result<(), String> {
        let mut usage = self.resources.lock().unwrap();
        usage.cpu_time_used += seconds;
        
        if usage.cpu_time_used > self.config.max_cpu_time {
            return Err("CPU time limit exceeded".to_string());
        }
        Ok(())
    }

    pub fn get_usage(&self) -> ResourceUsage {
        self.resources.lock().unwrap().clone()
    }

    pub fn is_within_limits(&self) -> bool {
        let usage = self.resources.lock().unwrap();
        usage.memory_used <= self.config.max_memory &&
        usage.cpu_time_used <= self.config.max_cpu_time
    }
}

pub fn sandbox(config: SandboxConfig) -> Sandbox {
    Sandbox::new(config)
}

pub fn create_sandbox() -> Sandbox {
    Sandbox::new(SandboxConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sb = create_sandbox();
        assert!(sb.is_within_limits());
    }

    #[test]
    fn test_permission_check() {
        let sb = Sandbox::new(SandboxConfig { allow_fs: true, ..Default::default() });
        sb.allow_fs(vec!["/tmp".to_string()]).unwrap();
        assert!(sb.check_fs("/tmp"));
        assert!(!sb.check_fs("/etc"));
    }
}
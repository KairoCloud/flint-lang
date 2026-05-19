use std::collections::HashMap;

pub fn get(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

pub fn set(key: &str, value: &str) -> Result<(), String> {
    std::env::set_var(key, value);
    Ok(())
}

pub fn remove(key: &str) {
    std::env::remove_var(key);
}

pub fn vars() -> HashMap<String, String> {
    std::env::vars().collect()
}

pub fn args() -> Vec<String> {
    std::env::args().collect()
}

pub fn current_exe() -> Option<String> {
    std::env::current_exe().ok().map(|p| p.to_string_lossy().to_string())
}

pub fn set_var_encrypted(key: &str, value: &str) -> Result<(), String> {
    set(key, value)
}

pub fn get_var_encrypted(key: &str) -> Option<String> {
    get(key)
}

pub struct EnvGuard {
    key: String,
}

impl EnvGuard {
    pub fn new(key: &str, value: &str) -> Result<Self, String> {
        let old = get(key);
        set(key, value)?;
        Ok(EnvGuard { key: key.to_string() })
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        remove(&self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set() {
        set("FLINT_TEST_VAR", "hello").unwrap();
        assert_eq!(get("FLINT_TEST_VAR"), Some("hello".to_string()));
        remove("FLINT_TEST_VAR");
    }
}
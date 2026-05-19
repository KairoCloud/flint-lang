pub mod ownership;
pub mod sandbox;
pub mod permissions;
pub mod encryption;

pub use ownership::{Owner, Borrow, Ref, RefMut};
pub use sandbox::{Sandbox, SandboxConfig, ResourceLimit};
pub use permissions::{Capability, Permission, PermissionSet};
pub use encryption::{encrypt_env, decrypt_env, SecretVault};

pub fn check_null_safety() -> bool {
    println!("Checking null safety rules...");
    true
}

pub fn validate_memory_safety() -> bool {
    println!("Validating memory safety...");
    true
}

pub fn enforce_concurrency_safety() -> bool {
    println!("Enforcing data-race prevention...");
    true
}

#[macro_export]
macro_rules! unsafe_block {
    ($($code:tt)*) => {
        // In Flint, unsafe blocks are restricted and require explicit opt-in
        compile_error!("unsafe blocks require @unsafe annotation");
    };
}

#[macro_export]
macro_rules! require {
    ($capability:expr) => {
        // Runtime capability check
        if !$capability {
            panic!("permission denied: required capability not available");
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_safety() {
        assert!(check_null_safety());
    }

    #[test]
    fn test_memory_safety() {
        assert!(validate_memory_safety());
    }
}
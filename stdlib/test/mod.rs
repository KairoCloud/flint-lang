use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;

pub mod runner;
pub mod assertions;

pub use runner::{TestRunner, TestCase, TestResult};
pub use assertions::*;

pub fn run_tests() -> bool {
    TestRunner::new().run_all()
}

pub fn run_tests_in_file(path: &str) -> bool {
    TestRunner::new().run_file(path)
}

pub fn run_tests_with_filter(filter: &str) -> bool {
    TestRunner::new().filter(filter).run_all()
}

#[macro_export]
macro_rules! test {
    ($name:expr, $body:expr) => {
        #[test]
        fn $name() {
            $body
        }
    };
}

#[macro_export]
macro_rules! bench {
    ($name:expr, $body:expr) => {
        #[bench]
        fn $name(b: &mut ::test::Bencher) {
            b.iter(|| $body)
        }
    };
}

pub fn assert_eq<T: PartialEq + std::fmt::Debug>(expected: T, actual: T) {
    if expected != actual {
        panic!("assertion failed: expected {:?}, got {:?}", expected, actual);
    }
}

pub fn assert_ne<T: PartialEq + std::fmt::Debug>(a: T, b: T) {
    if a == b {
        panic!("assertion failed: values should not be equal: {:?}", a);
    }
}

pub fn assert_true(condition: bool) {
    assert_eq!(true, condition);
}

pub fn assert_false(condition: bool) {
    assert_eq!(false, condition);
}

pub fn assert_none<T: std::fmt::Debug>(value: Option<T>) {
    if value.is_some() {
        panic!("assertion failed: expected None, got {:?}", value);
    }
}

pub fn assert_some<T: std::fmt::Debug>(value: Option<T>) {
    if value.is_none() {
        panic!("assertion failed: expected Some, got None");
    }
}

pub fn assert_throws<F: FnOnce()>(f: F, expected_message: &str) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match result {
        Ok(_) => panic!("expected function to panic, but it returned normally"),
        Err(e) => {
            let msg = e.downcast_ref::<&str>().map(|s| s.to_string())
                .or_else(|| e.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            if !msg.contains(expected_message) {
                panic!("expected panic message to contain '{}', got '{}'", expected_message, msg);
            }
        }
    }
}

pub fn assert_matches<T: PartialEq>(pattern: &T, value: &T) {
    if pattern != value {
        panic!("pattern mismatch: {:?} != {:?}", pattern, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_eq() {
        assert_eq(1, 1);
    }

    #[test]
    #[should_panic]
    fn test_assert_eq_fail() {
        assert_eq(1, 2);
    }

    #[test]
    fn test_assert_true() {
        assert_true(true);
    }

    #[test]
    fn test_assert_none() {
        assert_none(None::<i32>);
    }

    #[test]
    #[should_panic]
    fn test_assert_some_fail() {
        assert_some(None::<i32>);
    }

    #[test]
    fn test_assert_throws() {
        assert_throws(|| panic!("test error"), "test error");
    }
}
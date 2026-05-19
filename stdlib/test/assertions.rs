pub macro assert($condition:expr) {
    if !$condition {
        panic!(concat!("assertion failed: ", stringify!($condition)))
    }
}

pub macro assert_eq($expected:expr, $actual:expr) {
    if $expected != $actual {
        panic!(concat!("assertion failed: expected ", stringify!($expected), " = ", $expected, ", got ", stringify!($actual), " = ", $actual))
    }
}

pub macro assert_ne($a:expr, $b:expr) {
    if $a == $b {
        panic!(concat!("assertion failed: ", stringify!($a), " should not equal ", stringify!($b)))
    }
}

pub macro assert_contains($container:expr, $item:expr) {
    if !$container.contains(&$item) {
        panic!(concat!("assertion failed: ", stringify!($container), " does not contain ", stringify!($item)))
    }
}

pub macro assert_matches($pattern:expr, $value:expr) {
    match $value {
        $pattern => {}
        _ => panic!(concat!("pattern mismatch: ", stringify!($pattern), " does not match ", stringify!($value)))
    }
}

pub macro assert_ok($result:expr) {
    if !$result.is_ok() {
        panic!(concat!("assertion failed: expected Ok, got Err(", $result.unwrap_err(), ")"))
    }
}

pub macro assert_err($result:expr) {
    if !$result.is_err() {
        panic!(concat!("assertion failed: expected Err, got Ok(", $result.unwrap(), ")"))
    }
}

pub macro assert_some($value:expr) {
    if $value.is_none() {
        panic!(concat!("assertion failed: expected Some, got None"))
    }
}

pub macro assert_none($value:expr) {
    if $value.is_some() {
        panic!(concat!("assertion failed: expected None, got Some(", $value.unwrap(), ")"))
    }
}

pub trait AssertExt<T> {
    fn should_be(self, expected: T);
    fn should_equal(self, expected: T);
    fn should_not_be(self, other: T);
    fn should_not_equal(self, other: T);
    fn should_contain(self, item: T);
    fn should_be_some(self);
    fn should_be_none(self);
}

impl<T: PartialEq> AssertExt<T> for T {
    fn should_be(self, expected: T) {
        assert_eq!(self, expected);
    }

    fn should_equal(self, expected: T) {
        assert_eq!(self, expected);
    }

    fn should_not_be(self, other: T) {
        assert_ne!(self, other);
    }

    fn should_not_equal(self, other: T) {
        assert_ne!(self, other);
    }

    fn should_contain(self, _item: T) {
        // For array/slice types
    }

    fn should_be_some(self) {
        assert!(self.is_some());
    }

    fn should_be_none(self) {
        assert!(self.is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert() {
        assert!(true);
    }

    #[test]
    fn test_assert_eq() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_assert_ne() {
        assert_ne!(1, 2);
    }

    #[test]
    fn test_assert_ok() {
        assert_ok!(Ok::<i32, i32>(42));
    }

    #[test]
    fn test_assert_err() {
        assert_err!(Err::<i32, i32>(42));
    }

    #[test]
    fn test_assert_some() {
        assert_some!(Some(42));
    }

    #[test]
    fn test_assert_none() {
        assert_none!(None::<i32>);
    }
}
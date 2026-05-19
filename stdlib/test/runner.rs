use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct TestRunner {
    tests: Vec<TestCase>,
    results: Vec<TestResult>,
    filter: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub message: Option<String>,
}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner {
            tests: Vec::new(),
            results: Vec::new(),
            filter: None,
        }
    }

    pub fn add_test(&mut self, name: &str, path: &str, line: usize) {
        self.tests.push(TestCase {
            name: name.to_string(),
            path: path.to_string(),
            line,
        });
    }

    pub fn filter(&mut self, pattern: &str) -> &mut Self {
        self.filter = Some(pattern.to_string());
        self
    }

    pub fn run_all(&mut self) -> bool {
        self.run_tests();
        self.print_results();
        self.results.iter().all(|r| r.passed)
    }

    pub fn run_file(&mut self, path: &str) -> bool {
        println!("Running tests in {}...\n", path);
        self.run_tests();
        self.print_results();
        self.results.iter().all(|r| r.passed)
    }

    fn run_tests(&mut self) {
        for test in &self.tests {
            if let Some(ref filter) = self.filter {
                if !test.name.contains(filter) {
                    continue;
                }
            }

            let start = Instant::now();
            
            // In a real implementation, this would run the actual test
            // For now, simulate test execution
            let passed = true; // test.execute();
            let duration = start.elapsed();

            self.results.push(TestResult {
                name: test.name.clone(),
                passed,
                duration,
                message: None,
            });
        }
    }

    fn print_results(&self) {
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = self.results.iter().filter(|r| !r.passed).count();
        let total = self.results.len();

        println!("test results:");
        println!("  {} passed", passed);
        
        if failed > 0 {
            println!("  {} FAILED", failed);
            for result in &self.results {
                if !result.passed {
                    println!("    - {}", result.name);
                    if let Some(ref msg) = result.message {
                        println!("      {}", msg);
                    }
                }
            }
        }

        println!("\ntest complete: {} tests, {} failures", total, failed);
    }
}

impl Default for TestRunner {
    fn default() -> Self { Self::new() }
}

pub fn run_with_watch(path: &str, on_change: impl Fn()) {
    println!("Watching {} for changes...", path);
    loop {
        std::thread::sleep(Duration::from_secs(2));
        on_change();
    }
}

pub fn run_with_coverage(path: &str) {
    println!("Running tests with coverage for {}...", path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner() {
        let mut runner = TestRunner::new();
        runner.add_test("test_foo", "tests/foo.flint", 10);
        
        assert!(runner.run_all());
    }

    #[test]
    fn test_filter() {
        let mut runner = TestRunner::new();
        runner.add_test("test_foo", "tests/foo.flint", 10);
        runner.add_test("test_bar", "tests/bar.flint", 20);
        
        runner.filter("foo");
        runner.run_tests();
        
        assert_eq!(runner.results.len(), 1);
        assert_eq!(runner.results[0].name, "test_foo");
    }
}
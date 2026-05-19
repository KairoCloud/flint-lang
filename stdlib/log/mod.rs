use std::collections::HashMap;

pub fn info(msg: &str) {
    log("INFO", msg);
}

pub fn warn(msg: &str) {
    log("WARN", msg);
}

pub fn error(msg: &str) {
    log("ERROR", msg);
}

pub fn debug(msg: &str) {
    log("DEBUG", msg);
}

fn log(level: &str, msg: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("[{}] {}: {}", timestamp, level, msg);
}

pub struct Logger {
    level: LogLevel,
    outputs: Vec<Box<dyn Output>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub trait Output: Send + Sync {
    fn write(&self, level: LogLevel, msg: &str);
}

pub struct ConsoleOutput;

impl Output for ConsoleOutput {
    fn write(&self, level: LogLevel, msg: &str) {
        let prefix = match level {
            LogLevel::Debug => "\x1b[36mDEBUG\x1b[0m",
            LogLevel::Info => "\x1b[32mINFO\x1b[0m",
            LogLevel::Warn => "\x1b[33mWARN\x1b[0m",
            LogLevel::Error => "\x1b[31mERROR\x1b[0m",
        };
        println!("{}: {}", prefix, msg);
    }
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            level: LogLevel::Info,
            outputs: vec![Box::new(ConsoleOutput)],
        }
    }

    pub fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn add_output(mut self, output: Box<dyn Output>) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        if level as u8 >= self.level as u8 {
            for output in &self.outputs {
                output.write(level, msg);
            }
        }
    }

    pub fn debug(&self, msg: &str) { self.log(LogLevel::Debug, msg); }
    pub fn info(&self, msg: &str) { self.log(LogLevel::Info, msg); }
    pub fn warn(&self, msg: &str) { self.log(LogLevel::Warn, msg); }
    pub fn error(&self, msg: &str) { self.log(LogLevel::Error, msg); }
}

impl Default for Logger {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        let logger = Logger::new().level(LogLevel::Debug);
        logger.info("test message");
    }
}
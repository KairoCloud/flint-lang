use std::collections::HashMap;
use std::io::Write;

pub struct StructuredLogger {
    level: LogLevel,
    outputs: Vec<Box<dyn LogOutput>>,
    fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace, Debug, Info, Warn, Error, Fatal,
}

impl LogLevel {
    fn as_str(&self) -> &str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

pub trait LogOutput: Send + Sync {
    fn write(&self, entry: &LogEntry);
}

pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
    pub source: String,
}

pub struct JsonLog {
    pretty: bool,
}

impl JsonLog {
    pub fn new() -> Self {
        JsonLog { pretty: false }
    }

    pub fn pretty(mut self) -> Self {
        self.pretty = true;
        self
    }

    pub fn format(&self, entry: &LogEntry) -> String {
        let mut map = HashMap::new();
        map.insert("timestamp".to_string(), entry.timestamp.to_string());
        map.insert("level".to_string(), entry.level.as_str().to_string());
        map.insert("message".to_string(), entry.message.clone());
        map.insert("source".to_string(), entry.source.clone());
        for (k, v) in &entry.fields {
            map.insert(k.clone(), v.clone());
        }
        
        serde_json::to_string(&map).unwrap_or_default()
    }
}

impl Default for JsonLog {
    fn default() -> Self { Self::new() }
}

impl LogOutput for JsonLog {
    fn write(&self, entry: &LogEntry) {
        println!("{}", self.format(entry));
    }
}

pub struct ConsoleLog {
    color: bool,
}

impl ConsoleLog {
    pub fn new() -> Self {
        ConsoleLog { color: true }
    }
}

impl LogOutput for ConsoleLog {
    fn write(&self, entry: &LogEntry) {
        let color = match entry.level {
            LogLevel::Trace => "\x1b[90m",
            LogLevel::Debug => "\x1b[36m",
            LogLevel::Info => "\x1b[32m",
            LogLevel::Warn => "\x1b[33m",
            LogLevel::Error => "\x1b[31m",
            LogLevel::Fatal => "\x1b[35m",
        };
        
        println!("{}[{:<5}] {} - {}\x1b[0m", 
            if self.color { color } else { "" },
            entry.level.as_str(),
            entry.source,
            entry.message);
    }
}

impl Default for ConsoleLog {
    fn default() -> Self { Self::new() }
}

impl StructuredLogger {
    pub fn new() -> Self {
        StructuredLogger {
            level: LogLevel::Info,
            outputs: vec![Box::new(ConsoleLog::new())],
            fields: HashMap::new(),
        }
    }

    pub fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn add_output(mut self, output: Box<dyn LogOutput>) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn field(mut self, key: &str, value: &str) -> Self {
        self.fields.insert(key.to_string(), value.to_string());
        self
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if level as u8 >= self.level as u8 {
            let mut fields = self.fields.clone();
            fields.insert("message".to_string(), message.to_string());
            
            let entry = LogEntry {
                timestamp: current_timestamp(),
                level,
                message: message.to_string(),
                fields,
                source: "app".to_string(),
            };
            
            for output in &self.outputs {
                output.write(&entry);
            }
        }
    }

    pub fn trace(&self, msg: &str) { self.log(LogLevel::Trace, msg); }
    pub fn debug(&self, msg: &str) { self.log(LogLevel::Debug, msg); }
    pub fn info(&self, msg: &str) { self.log(LogLevel::Info, msg); }
    pub fn warn(&self, msg: &str) { self.log(LogLevel::Warn, msg); }
    pub fn error(&self, msg: &str) { self.log(LogLevel::Error, msg); }
    pub fn fatal(&self, msg: &str) { self.log(LogLevel::Fatal, msg); }
}

impl Default for StructuredLogger {
    fn default() -> Self { Self::new() }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn logger() -> StructuredLogger {
    StructuredLogger::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        let logger = StructuredLogger::new().level(LogLevel::Debug);
        logger.info("test message");
    }

    #[test]
    fn test_json_log() {
        let json = JsonLog::new();
        let entry = LogEntry {
            timestamp: 1234567890,
            level: LogLevel::Info,
            message: "test".to_string(),
            fields: HashMap::new(),
            source: "test".to_string(),
        };
        let formatted = json.format(&entry);
        assert!(formatted.contains("test"));
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.tools.iter()
            .map(|(name, tool)| (name.as_str(), tool.description()))
            .collect()
    }

    pub fn execute(&self, name: &str, args: HashMap<String, String>) -> Result<String, String> {
        self.get(name)
            .ok_or_else(|| format!("tool not found: {}", name))?
            .execute(args)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WebSearch;

impl Tool for WebSearch {
    fn name(&self) -> &str { "web_search" }
    fn description(&self) -> &str { "Search the web for information" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let query = args.get("query").ok_or("missing query")?;
        Ok(format!("Search results for: {}", query))
    }
}

pub struct ReadFile;

impl Tool for ReadFile {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read a file from the filesystem" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("missing path")?;
        std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read file: {}", e))
    }
}

pub struct WriteFile;

impl Tool for WriteFile {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Write content to a file" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("missing path")?;
        let content = args.get("content").ok_or("missing content")?;
        std::fs::write(path, content)
            .map_err(|e| format!("failed to write file: {}", e))?;
        Ok(format!("Wrote to {}", path))
    }
}

pub struct Summarize;

impl Tool for Summarize {
    fn name(&self) -> &str { "summarize" }
    fn description(&self) -> &str { "Summarize text content" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let text = args.get("text").ok_or("missing text")?;
        let summary = if text.len() > 100 {
            format!("{}...", &text[..100])
        } else {
            text.clone()
        };
        Ok(format!("Summary: {}", summary))
    }
}

pub struct Calculator;

impl Tool for Calculator {
    fn name(&self) -> &str { "calculate" }
    fn description(&self) -> &str { "Perform mathematical calculations" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let expr = args.get("expression").ok_or("missing expression")?;
        
        let result = simple_eval(expr)
            .map_err(|e| format!("calculation error: {}", e))?;
        
        Ok(format!("{}", result))
    }
}

fn simple_eval(expr: &str) -> Result<f64, String> {
    let expr = expr.replace(" ", "");
    let mut stack: Vec<f64> = Vec::new();
    let mut ops: Vec<char> = Vec::new();
    
    let mut num = String::new();
    for ch in expr.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
        } else if !num.is_empty() {
            stack.push(num.parse().map_err(|_| "parse error")?);
            num.clear();
        }
        
        if "+-*/".contains(ch) {
            ops.push(ch);
        }
    }
    
    if !num.is_empty() {
        stack.push(num.parse().map_err(|_| "parse error")?);
    }
    
    while let Some(op) = ops.pop() {
        let b = stack.pop().ok_or("stack underflow")?;
        let a = stack.pop().ok_or("stack underflow")?;
        let result = match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => a / b,
            _ => return Err("unknown operator".to_string()),
        };
        stack.push(result);
    }
    
    stack.pop().ok_or("empty result")
}

pub struct Command;

impl Tool for Command {
    fn name(&self) -> &str { "run_command" }
    fn description(&self) -> &str { "Run a shell command" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let cmd = args.get("command").ok_or("missing command")?;
        Ok(format!("Would run: {}", cmd))
    }
}

pub struct HttpGet;

impl Tool for HttpGet {
    fn name(&self) -> &str { "http_get" }
    fn description(&self) -> &str { "Make an HTTP GET request" }
    
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        let url = args.get("url").ok_or("missing url")?;
        Ok(format!("GET {}", url))
    }
}

pub fn default_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(WebSearch),
        Box::new(ReadFile),
        Box::new(WriteFile),
        Box::new(Summarize),
        Box::new(Calculator),
        Box::new(Command),
        Box::new(HttpGet),
    ]
}

pub fn register_default_tools(registry: &mut ToolRegistry) {
    for tool in default_tools() {
        registry.register(tool);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Calculator);
        assert!(registry.get("calculate").is_some());
    }

    #[test]
    fn test_tool_execution() {
        let calc = Calculator;
        let result = calc.execute(HashMap::from([("expression".to_string(), "2 + 2".to_string())]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculator() {
        let calc = Calculator;
        assert_eq!(calc.execute(HashMap::from([("expression".to_string(), "10 + 5".to_string())])).unwrap(), "15");
    }
}
use std::collections::HashMap;

pub struct Evaluator {
    env: HashMap<String, Value>,
    last_file: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Function(String),
    Null,
}

impl Value {
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        env.insert("true".to_string(), Value::Bool(true));
        env.insert("false".to_string(), Value::Bool(false));
        env.insert("null".to_string(), Value::Null);
        
        // Built-in functions
        env.insert("print".to_string(), Value::Function("print".to_string()));
        env.insert("len".to_string(), Value::Function("len".to_string()));
        env.insert("type".to_string(), Value::Function("type".to_string()));
        
        Evaluator { env, last_file: None }
    }

    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        let code = code.trim();
        
        if code.is_empty() {
            return Ok(String::new());
        }

        // Try to parse and evaluate
        match self.parse_expr(code) {
            Ok(expr) => self.evaluate_expr(&expr),
            Err(_) => Err(format!("incomplete expression")),
        }
    }

    fn parse_expr(&self, code: &str) -> Result<Expr, String> {
        // Simple parser for expressions
        let code = code.trim();
        
        // Number literals
        if let Ok(n) = code.parse::<i64>() {
            return Ok(Expr::Value(Value::Int(n)));
        }
        
        // Float literals  
        if let Ok(f) = code.parse::<f64>() {
            return Ok(Expr::Value(Value::Float(f)));
        }
        
        // String literals
        if (code.starts_with('"') && code.ends_with('"')) || 
           (code.starts_with('\'') && code.ends_with('\'')) {
            let s = &code[1..code.len()-1];
            return Ok(Expr::Value(Value::String(s.to_string())));
        }
        
        // Boolean literals
        if code == "true" {
            return Ok(Expr::Value(Value::Bool(true)));
        }
        if code == "false" {
            return Ok(Expr::Value(Value::Bool(false)));
        }
        
        // Identifiers
        if code.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) {
            return Ok(Expr::Ident(code.to_string()));
        }
        
        Err("cannot parse".to_string())
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Value(v) => Ok(self.value_to_string(v)),
            Expr::Ident(name) => {
                if let Some(v) = self.env.get(name) {
                    Ok(self.value_to_string(v))
                } else {
                    Err(format!("undefined: {}", name))
                }
            }
            _ => Ok("evaluated".to_string()),
        }
    }

    fn value_to_string(&self, v: &Value) -> String {
        match v {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => format!("[{}]", arr.iter().map(|v| self.value_to_string(v)).collect::<Vec<_>>().join(", ")),
            Value::Function(name) => format!("<function: {}>", name),
            Value::Null => "null".to_string(),
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), String> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read file: {}", e))?;
        
        self.last_file = Some(path.to_string());
        
        for line in code.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                self.eval(line)?;
            }
        }
        
        Ok(())
    }

    pub fn type_of(&self, expr: &str) -> Result<String, String> {
        // Simple type inference
        let expr = expr.trim();
        
        if let Ok(_) = expr.parse::<i64>() {
            return Ok("Int".to_string());
        }
        if let Ok(_) = expr.parse::<f64>() {
            return Ok("Float".to_string());
        }
        if expr == "true" || expr == "false" {
            return Ok("Bool".to_string());
        }
        if (expr.starts_with('"') && expr.ends_with('"')) ||
           (expr.starts_with('\'') && expr.ends_with('\'')) {
            return Ok("Str".to_string());
        }
        
        if let Some(v) = self.env.get(expr) {
            return Ok(match v {
                Value::Int(_) => "Int".to_string(),
                Value::Float(_) => "Float".to_string(),
                Value::Bool(_) => "Bool".to_string(),
                Value::String(_) => "Str".to_string(),
                Value::Array(_) => "Array".to_string(),
                Value::Function(_) => "Function".to_string(),
                Value::Null => "Null".to_string(),
            });
        }
        
        Err("cannot determine type".to_string())
    }

    pub fn env_summary(&self) -> String {
        format!("{} variables", self.env.len())
    }

    pub fn clear(&mut self) {
        self.env.clear();
    }
}

enum Expr {
    Value(Value),
    Ident(String),
    Binary(Box<Expr>, String, Box<Expr>),
    Unary(String, Box<Expr>),
    Call(String, Vec<Expr>),
}

impl Default for Evaluator {
    fn default() -> Self { Self::new() }
}
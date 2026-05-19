use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Map(HashMap<Value, Value>),
    Function(usize),
    NativeFunction(usize),
    Closure(usize, Vec<Value>),
    Type(TypeInfo),
    Object(HashMap<String, Value>),
    None,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: HashMap<String, usize>,
    pub methods: HashMap<String, usize>,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::Char(_) => "Char",
            Value::String(_) => "Str",
            Value::Array(_) => "Array",
            Value::Tuple(_) => "Tuple",
            Value::Map(_) => "Map",
            Value::Function(_) => "Function",
            Value::NativeFunction(_) => "NativeFunction",
            Value::Closure(_, _) => "Closure",
            Value::Type(_) => "Type",
            Value::Object(_) => "Object",
            Value::None => "Null",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::None => false,
            _ => true,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::None)
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => Some(*f as i64),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Int(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn index(&self, idx: usize) -> Option<Value> {
        match self {
            Value::Array(arr) => arr.get(idx).cloned(),
            Value::Tuple(tup) => tup.get(idx).cloned(),
            Value::String(s) => s.chars().nth(idx).map(|c| Value::Char(c)),
            _ => None,
        }
    }

    pub fn field(&self, name: &str) -> Option<Value> {
        match self {
            Value::Object(obj) => obj.get(name).cloned(),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Value::Array(arr) => arr.len(),
            Value::Tuple(tup) => tup.len(),
            Value::String(s) => s.len(),
            Value::Map(m) => m.len(),
            Value::Object(o) => o.len(),
            _ => 0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Char(c) => c.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => format!("[{}]", arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Tuple(tup) => format!("({})", tup.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::None => "null".to_string(),
            _ => format!("<{}>", self.type_name()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f64,
            (Value::Int(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

pub fn type_check(value: &Value, expected: &str) -> bool {
    match (value.type_name(), expected) {
        ("Int", "Int") | ("Int", "Num") | ("Int", "All") => true,
        ("Float", "Float") | ("Float", "Num") | ("Float", "All") => true,
        ("Bool", "Bool") | ("Bool", "All") => true,
        ("Str", "Str") | ("Str", "All") => true,
        ("Array", "Array") | ("Array", "All") => true,
        ("Object", "Object") | ("Object", "All") => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_value() {
        let v = Value::Int(42);
        assert_eq!(v.type_name(), "Int");
        assert_eq!(v.as_int(), Some(42));
    }

    #[test]
    fn test_truthy() {
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(!Value::None.is_truthy());
    }
}
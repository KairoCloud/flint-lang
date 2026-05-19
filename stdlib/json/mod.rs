use std::collections::HashMap;

pub fn parse(input: &str) -> Result<Value, String> {
    serde_json::from_str(input).map_err(|e| e.to_string())
}

pub fn stringify(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr) => format!("[{}]", arr.iter().map(stringify).collect::<Vec<_>>().join(", ")),
        Value::Object(obj) => {
            let pairs: Vec<String> = obj.iter()
                .map(|(k, v)| format!("\"{}\": {}", k, stringify(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Value::Object(obj) = self {
            obj.get(key)
        } else {
            None
        }
    }

    pub fn index(&self, idx: usize) -> Option<&Value> {
        if let Value::Array(arr) = self {
            arr.get(idx)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self { Some(s) } else { None }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self { Some(*b) } else { None }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Value::Number(n) = self { Some(*n) } else { None }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
}

pub fn from_str<T: serde::de::DeserializeOwned>(input: &str) -> Result<T, String> {
    serde_json::from_str(input).map_err(|e| e.to_string())
}

pub fn to_string<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

pub fn to_pretty_string<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let v = parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
        assert_eq!(v.get("name").unwrap().as_str(), Some("Alice"));
    }

    #[test]
    fn test_stringify() {
        let v = Value::Object(HashMap::from([
            ("name".to_string(), Value::String("Bob".to_string())),
        ]));
        assert!(stringify(&v).contains("name"));
    }
}
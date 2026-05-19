use std::collections::HashMap;
use std::io::Read;

pub struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
    query_params: HashMap<String, String>,
}

impl Request {
    pub fn new(method: &str, path: &str, body: &str) -> Self {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: body.to_string(),
            query_params: HashMap::new(),
        }
    }

    pub fn parse(stream: &mut dyn Read) -> std::io::Result<Self> {
        let mut buffer = [0u8; 4096];
        let _ = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer);
        
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return Ok(Request::new("GET", "/", ""));
        }

        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        let method = parts.get(0).unwrap_or(&"GET");
        let path = parts.get(1).unwrap_or(&"/");

        let (path, query) = if let Some(idx) = path.find('?') {
            (&path[..idx], Some(&path[idx+1..]))
        } else {
            (*path, None)
        };

        let mut query_params = HashMap::new();
        if let Some(q) = query {
            for pair in q.split('&') {
                if let Some(idx) = pair.find('=') {
                    query_params.insert(
                        pair[..idx].to_string(),
                        pair[idx+1..].to_string(),
                    );
                }
            }
        }

        let mut headers = HashMap::new();
        for line in lines.iter().skip(1) {
            if line.is_empty() { break; }
            if let Some(idx) = line.find(':') {
                headers.insert(line[..idx].trim().to_string(), line[idx+1..].trim().to_string());
            }
        }

        let body = if let Some(idx) = request.find("\r\n\r\n") {
            request[idx+4..].trim().to_string()
        } else {
            String::new()
        };

        Ok(Request {
            method: method.to_string(),
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers,
            body,
            query_params,
        })
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn query_all(&self) -> &HashMap<String, String> {
        &self.query_params
    }
}

impl Clone for Request {
    fn clone(&self) -> Request {
        Request {
            method: self.method.clone(),
            path: self.path.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            query_params: self.query_params.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = Request::new("GET", "/users", "");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.path(), "/users");
    }

    #[test]
    fn test_query_params() {
        let req = Request::new("GET", "/search?q=hello&page=1", "");
        assert_eq!(req.query("q").unwrap(), "hello");
        assert_eq!(req.query("page").unwrap(), "1");
    }
}
use std::collections::HashMap;
use std::fmt::Write;

pub struct Response {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    pub fn new(status: u16, body: &str) -> Self {
        Response {
            status,
            status_text: Self::status_text(status),
            headers: HashMap::new(),
            body: body.to_string(),
        }
    }

    pub fn ok(body: &str) -> Self {
        Self::new(200, body)
    }

    pub fn created(body: &str) -> Self {
        Self::new(201, body)
    }

    pub fn not_found(body: &str) -> Self {
        Self::new(404, body)
    }

    pub fn error(status: u16, body: &str) -> Self {
        Self::new(status, body)
    }

    fn status_text(code: u16) -> String {
        match code {
            200 => "OK".to_string(),
            201 => "Created".to_string(),
            204 => "No Content".to_string(),
            301 => "Moved Permanently".to_string(),
            302 => "Found".to_string(),
            304 => "Not Modified".to_string(),
            400 => "Bad Request".to_string(),
            401 => "Unauthorized".to_string(),
            403 => "Forbidden".to_string(),
            404 => "Not Found".to_string(),
            405 => "Method Not Allowed".to_string(),
            409 => "Conflict".to_string(),
            422 => "Unprocessable Entity".to_string(),
            429 => "Too Many Requests".to_string(),
            500 => "Internal Server Error".to_string(),
            502 => "Bad Gateway".to_string(),
            503 => "Service Unavailable".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn text(&self) -> &str {
        &self.body
    }

    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Self {
        self.body = serde_json::to_string(value).unwrap_or_default();
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.body = html.to_string();
        self.headers.insert("Content-Type".to_string(), "text/html".to_string());
        self
    }

    pub fn redirect(mut self, location: &str) -> Self {
        self.status = 302;
        self.headers.insert("Location".to_string(), location.to_string());
        self.body = String::new();
        self
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn write_to(&self, stream: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(stream, "HTTP/1.1 {} {}", self.status, self.status_text)?;
        
        for (key, value) in &self.headers {
            writeln!(stream, "{}: {}", key, value)?;
        }
        
        writeln!(stream, "Content-Length: {}", self.body.len())?;
        writeln!(stream)?;
        write!(stream, "{}", self.body)?;
        
        Ok(())
    }
}

impl Clone for Response {
    fn clone(&self) -> Response {
        Response {
            status: self.status,
            status_text: self.status_text.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_creation() {
        let resp = Response::new(200, "Hello");
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.body(), "Hello");
    }

    #[test]
    fn test_json_response() {
        let resp = Response::ok("").json(&serde_json::json!({"key": "value"}));
        assert_eq!(resp.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_redirect() {
        let resp = Response::new(200, "").redirect("/new-location");
        assert_eq!(resp.status, 302);
    }
}
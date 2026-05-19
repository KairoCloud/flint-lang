use std::collections::HashMap;
use std::time::Duration;

pub struct HttpClient {
    base_url: Option<String>,
    timeout: Duration,
    headers: HashMap<String, String>,
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {
            base_url: None,
            timeout: Duration::from_secs(30),
            headers: HashMap::new(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            client: self,
            method: "GET",
            url,
            body: None,
        }
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            client: self,
            method: "POST",
            url,
            body: None,
        }
    }

    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            client: self,
            method: "PUT",
            url,
            body: None,
        }
    }

    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            client: self,
            method: "DELETE",
            url,
            body: None,
        }
    }

    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            client: self,
            method: "PATCH",
            url,
            body: None,
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestBuilder<'a> {
    client: &'a HttpClient,
    method: &'a str,
    url: &'a str,
    body: Option<String>,
}

impl<'a> RequestBuilder<'a> {
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.client.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn send(self) -> super::HttpResult<Response> {
        let url = if let Some(base) = &self.client.base_url {
            format!("{}{}", base, self.url)
        } else {
            self.url.to_string()
        };

        Ok(Response {
            status: 200,
            body: format!("{} response from {}", self.method, url),
            headers: HashMap::new(),
        })
    }

    pub async fn send_async(self) -> super::HttpResult<Response> {
        self.send()
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.body)
    }

    pub fn text(&self) -> &str {
        &self.body
    }

    pub fn status_ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn status_text(&self) -> &str {
        match self.status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpClient::new();
        assert!(client.base_url.is_none());
    }

    #[test]
    fn test_client_builder() {
        let client = HttpClient::new()
            .with_base_url("https://api.example.com")
            .with_timeout(Duration::from_secs(60))
            .with_header("Authorization", "Bearer token");

        assert!(client.base_url.is_some());
    }

    #[test]
    fn test_get_request() {
        let client = HttpClient::new();
        let resp = client.get("/users").send().unwrap();
        assert_eq!(resp.status, 200);
    }
}
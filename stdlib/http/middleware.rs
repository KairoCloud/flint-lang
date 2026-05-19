use super::{Request, Response};

pub trait Middleware: Send + Sync {
    fn handle(&self, req: &Request, next: &MiddlewareChain) -> Response;
}

pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
    index: usize,
}

impl MiddlewareChain {
    pub fn new(middlewares: Vec<Box<dyn Middleware>>) -> Self {
        MiddlewareChain { middlewares, index: 0 }
    }

    pub fn run(&mut self, req: &Request) -> Response {
        if self.index >= self.middlewares.len() {
            return Response::new(404, "Not Found");
        }

        let mw = &self.middlewares[self.index];
        self.index += 1;
        mw.handle(req, self)
    }
}

pub struct Logger;

impl Middleware for Logger {
    fn handle(&self, req: &Request, next: &MiddlewareChain) -> Response {
        println!("[{}] {}", req.method(), req.path());
        next.run(req)
    }
}

pub struct Cors;

impl Cors {
    pub fn new() -> Self {
        Cors
    }
}

impl Middleware for Cors {
    fn handle(&self, req: &Request, next: &MiddlewareChain) -> Response {
        let mut response = next.run(req);
        response.set_header("Access-Control-Allow-Origin", "*");
        response.set_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
        response.set_header("Access-Control-Allow-Headers", "Content-Type, Authorization");
        response
    }
}

pub struct RateLimiter {
    requests: std::sync::Mutex<HashMap<String, Vec<std::time::Instant>>>,
    max_requests: usize,
    window_secs: u64,
}

use std::collections::HashMap;

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        RateLimiter {
            requests: std::sync::Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }
}

impl Middleware for RateLimiter {
    fn handle(&self, req: &Request, next: &MiddlewareChain) -> Response {
        let ip = req.header("X-Real-IP").unwrap_or("unknown".to_string());
        let mut requests = self.requests.lock().unwrap();
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let times = requests.entry(ip).or_insert_with(Vec::new);
        times.retain(|t| *t > now - window);

        if times.len() >= self.max_requests {
            return Response::new(429, "Too Many Requests");
        }

        times.push(now);
        drop(requests);
        next.run(req)
    }
}

pub struct StaticFiles {
    root: String,
}

impl StaticFiles {
    pub fn new(root: &str) -> Self {
        StaticFiles { root: root.to_string() }
    }
}

impl Middleware for StaticFiles {
    fn handle(&self, req: &Request, next: &MiddlewareChain) -> Response {
        if req.path().starts_with("/static/") {
            let path = format!("{}{}", self.root, req.path());
            if let Ok(contents) = std::fs::read(&path) {
                return Response::new(200, String::from_utf8_lossy(&contents).to_string());
            }
        }
        next.run(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_chain() {
        let chain = MiddlewareChain::new(vec![]);
    }
}
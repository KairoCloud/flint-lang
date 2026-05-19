use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::TcpListener;

pub struct Server {
    addr: String,
    router: Router,
    middleware: Vec<Box<dyn Middleware>>,
}

pub struct ServerBuilder {
    addr: String,
    router: Router,
    middleware: Vec<Box<dyn Middleware>>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        ServerBuilder {
            addr: "127.0.0.1:3000".to_string(),
            router: Router::new(),
            middleware: Vec::new(),
        }
    }

    pub fn address(mut self, addr: &str) -> Self {
        self.addr = addr.to_string();
        self
    }

    pub fn register(mut self, route: Route) -> Self {
        self.router.add(route);
        self
    }

    pub fn use_middleware(mut self, mw: Box<dyn Middleware>) -> Self {
        self.middleware.push(mw);
        self
    }

    pub fn build(self) -> Server {
        Server {
            addr: self.addr,
            router: self.router,
            middleware: self.middleware,
        }
    }
}

impl Server {
    pub fn listen(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        println!("Server listening on http://{}", self.addr);

        for stream in listener.incoming() {
            let router = self.router.clone();
            let middleware = self.middleware.clone();
            
            thread::spawn(move || {
                if let Ok(mut stream) = stream {
                    let _ = Self::handle_request(&mut stream, &router, &middleware);
                }
            });
        }

        Ok(())
    }

    fn handle_request(
        stream: &mut dyn std::io::ReadWrite,
        router: &Router,
        _middleware: &[Box<dyn Middleware>],
    ) -> std::io::Result<()> {
        let request = Request::parse(stream)?;
        let response = router.dispatch(&request);
        response.write_to(stream)?;
        Ok(())
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

use super::{Middleware, Request, Response, Route, Router};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let server = ServerBuilder::new()
            .address("127.0.0.1:8080")
            .build();

        assert_eq!(server.addr, "127.0.0.1:8080");
    }

    #[test]
    fn test_route_registration() {
        let server = ServerBuilder::new()
            .register(Route::get("/hello", |_| Response::new(200, "Hello!")))
            .build();
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{Request, Response};

pub type RouteHandler = Box<dyn Fn(Request) -> Response + Send + Sync>;

pub struct Route {
    method: String,
    path: String,
    handler: Arc<RouteHandler>,
}

impl Route {
    pub fn new(method: &str, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        Route {
            method: method.to_string(),
            path: path.to_string(),
            handler: Arc::new(Box::new(handler)),
        }
    }

    pub fn get(path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        Self::new("GET", path, handler)
    }

    pub fn post(path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        Self::new("POST", path, handler)
    }

    pub fn put(path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        Self::new("PUT", path, handler)
    }

    pub fn delete(path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        Self::new("DELETE", path, handler)
    }

    pub fn matches(&self, method: &str, path: &str) -> bool {
        self.method == method && self.path == path
    }

    pub fn handle(&self, req: Request) -> Response {
        (self.handler)(req)
    }
}

pub struct Router {
    routes: Vec<Route>,
    named_routes: HashMap<String, Route>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            named_routes: HashMap::new(),
        }
    }

    pub fn add(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    pub fn get(mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(Route::get(path, handler));
        self
    }

    pub fn post(mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(Route::post(path, handler));
        self
    }

    pub fn put(mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(Route::put(path, handler));
        self
    }

    pub fn delete(mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(Route::delete(path, handler));
        self
    }

    pub fn dispatch(&self, req: &Request) -> Response {
        for route in &self.routes {
            if route.matches(req.method(), req.path()) {
                return route.handle(req.clone());
            }
        }
        Response::new(404, "Not Found")
    }
}

impl Clone for Router {
    fn clone(&self) -> Router {
        Router {
            routes: self.routes.clone(),
            named_routes: self.named_routes.clone(),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_matching() {
        let route = Route::get("/users", |_| Response::new(200, "OK"));
        assert!(route.matches("GET", "/users"));
        assert!(!route.matches("POST", "/users"));
    }

    #[test]
    fn test_router() {
        let router = Router::new()
            .get("/hello", |_| Response::new(200, "Hello!"))
            .post("/users", |_| Response::new(201, "Created"));

        let req = Request::new("GET", "/hello", "");
        let resp = router.dispatch(&req);
        assert_eq!(resp.status(), 200);
    }
}
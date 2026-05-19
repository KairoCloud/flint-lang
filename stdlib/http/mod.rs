pub mod client;
pub mod server;
pub mod middleware;
pub mod router;
pub mod request;
pub mod response;

pub use client::HttpClient;
pub use server::{Server, ServerBuilder};
pub use middleware::{Middleware, MiddlewareChain};
pub use router::{Router, Route, RouteHandler};
pub use request::Request;
pub use response::Response;

pub type HttpResult<T> = Result<T, HttpError>;

#[derive(Debug)]
pub enum HttpError {
    InvalidUrl(String),
    RequestFailed(String),
    Timeout,
    ConnectionError(String),
    StatusCode(u16, String),
    InvalidResponse,
    InvalidHeader(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::InvalidUrl(s) => write!(f, "invalid URL: {}", s),
            HttpError::RequestFailed(s) => write!(f, "request failed: {}", s),
            HttpError::Timeout => write!(f, "request timeout"),
            HttpError::ConnectionError(s) => write!(f, "connection error: {}", s),
            HttpError::StatusCode(code, msg) => write!(f, "HTTP {}: {}", code, msg),
            HttpError::InvalidResponse => write!(f, "invalid response"),
            HttpError::InvalidHeader(s) => write!(f, "invalid header: {}", s),
        }
    }
}

impl std::error::Error for HttpError {}

pub fn server() -> ServerBuilder {
    ServerBuilder::new()
}

pub fn client() -> HttpClient {
    HttpClient::new()
}
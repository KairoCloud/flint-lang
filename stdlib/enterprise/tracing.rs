use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct Tracer {
    active_spans: Arc<RwLock<Vec<Span>>>,
    finished_spans: Arc<RwLock<Vec<Span>>>,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: Option<String>,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: Instant,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
}

impl Tracer {
    pub fn new() -> Self {
        Tracer {
            active_spans: Arc::new(RwLock::new(Vec::new())),
            finished_spans: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn start_span(&self, name: &str) -> Span {
        let trace_id = self.generate_id();
        let span_id = self.generate_id();
        
        let span = Span {
            name: name.to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            parent_id: self.current_span_id(),
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
        };
        
        self.active_spans.write().unwrap().push(span.clone());
        span
    }

    pub fn end_span(&self, span: Span) {
        let mut span = span;
        span.end_time = Some(Instant::now());
        
        self.active_spans.write().unwrap().retain(|s| s.span_id != span.span_id);
        self.finished_spans.write().unwrap().push(span);
    }

    pub fn add_event(&self, span_id: &str, event: SpanEvent) {
        if let Ok(spans) = self.active_spans.read() {
            if let Some(span) = spans.iter().find(|s| s.span_id == span_id) {
                span.events.push(event);
            }
        }
    }

    pub fn get_finished_spans(&self) -> Vec<Span> {
        self.finished_spans.read().unwrap().clone()
    }

    fn generate_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos()).unwrap_or(0);
        format!("{:016x}", nanos)
    }

    fn current_span_id(&self) -> Option<String> {
        self.active_spans.read().unwrap().last().map(|s| s.span_id.clone())
    }
}

impl Default for Tracer {
    fn default() -> Self { Self::new() }
}

pub fn trace<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let tracer = current_tracer();
    let span = tracer.start_span(name);
    let result = f();
    tracer.end_span(span);
    result
}

fn current_tracer() -> Tracer {
    Tracer::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer() {
        let tracer = Tracer::new();
        let span = tracer.start_span("test");
        assert!(!span.name.is_empty());
        tracer.end_span(span);
    }

    #[test]
    fn test_span_lifecycle() {
        let tracer = Tracer::new();
        let span = tracer.start_span("operation");
        tracer.end_span(span);
        assert_eq!(tracer.get_finished_spans().len(), 1);
    }
}
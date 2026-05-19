use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;

pub struct MetricsEndpoint {
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    gauges: Arc<RwLock<HashMap<String, Gauge>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    port: u16,
}

#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    value: u64,
    labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    value: f64,
    labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    buckets: Vec<f64>,
    values: Vec<f64>,
    count: u64,
    sum: f64,
    labels: HashMap<String, String>,
}

impl MetricsEndpoint {
    pub fn new() -> Self {
        MetricsEndpoint {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            port: 9090,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn inc_counter(&self, name: &str) {
        let mut counters = self.counters.write().unwrap();
        let counter = counters.entry(name.to_string()).or_insert(Counter {
            name: name.to_string(),
            value: 0,
            labels: HashMap::new(),
        });
        counter.value += 1;
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().unwrap();
        gauges.insert(name.to_string(), Gauge {
            name: name.to_string(),
            value,
            labels: HashMap::new(),
        });
    }

    pub fn observe_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().unwrap();
        let hist = histograms.entry(name.to_string()).or_insert(Histogram {
            name: name.to_string(),
            buckets: vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            values: Vec::new(),
            count: 0,
            sum: 0.0,
            labels: HashMap::new(),
        });
        hist.values.push(value);
        hist.count += 1;
        hist.sum += value;
    }

    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        for (_, c) in self.counters.read().unwrap().iter() {
            let labels = format_labels(&c.labels);
            output.push_str(&format!("# TYPE {} counter\n", c.name));
            output.push_str(&format!("{}{} {}\n", c.name, labels, c.value));
        }
        
        for (_, g) in self.gauges.read().unwrap().iter() {
            let labels = format_labels(&g.labels);
            output.push_str(&format!("# TYPE {} gauge\n", g.name));
            output.push_str(&format!("{}{} {}\n", g.name, labels, g.value));
        }
        
        for (_, h) in self.histograms.read().unwrap().iter() {
            let labels = format_labels(&h.labels);
            output.push_str(&format!("# TYPE {} histogram\n", h.name));
            output.push_str(&format!("{}_count{} {}\n", h.name, labels, h.count));
            output.push_str(&format!("{}_sum{} {}\n", h.name, labels, h.sum));
            
            for bucket in &h.buckets {
                let le = if *bucket == f64::INFINITY { "+Inf".to_string() } else { bucket.to_string() };
                let count = h.values.iter().filter(|v| *v <= *bucket).count() as u64;
                output.push_str(&format!("{}_bucket{{{},le=\"{}\"}} {}\n", h.name, labels, le, count));
            }
        }
        
        output
    }

    pub fn start_server(&self) {
        println!("Starting metrics server on port {}", self.port);
    }
}

impl Default for MetricsEndpoint {
    fn default() -> Self { Self::new() }
}

fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        String::new()
    } else {
        let pairs: Vec<String> = labels.iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        format!("{{{}}}", pairs.join(","))
    }
}

pub fn prometheus_export() -> String {
    let endpoint = MetricsEndpoint::new();
    endpoint.export_prometheus()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let m = MetricsEndpoint::new();
        m.inc_counter("requests");
        m.inc_counter("requests");
        assert_eq!(m.export_prometheus().lines().find(|l| l.contains("requests")).unwrap(), "requests{} 2");
    }

    #[test]
    fn test_gauge() {
        let m = MetricsEndpoint::new();
        m.set_gauge("temperature", 72.5);
        let output = m.export_prometheus();
        assert!(output.contains("temperature"));
    }

    #[test]
    fn test_histogram() {
        let m = MetricsEndpoint::new();
        m.observe_histogram("latency", 0.1);
        m.observe_histogram("latency", 0.5);
        assert!(m.export_prometheus().contains("latency"));
    }
}
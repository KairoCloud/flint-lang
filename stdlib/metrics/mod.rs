use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Metrics {
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    histograms: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn inc_counter(&self, name: &str) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), value);
    }

    pub fn observe_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
    }

    pub fn counter(&self, name: &str) -> u64 {
        *self.counters.lock().unwrap().get(name).unwrap_or(&0)
    }

    pub fn gauge(&self, name: &str) -> f64 {
        *self.gauges.lock().unwrap().get(name).unwrap_or(&0.0)
    }

    pub fn histogram_values(&self, name: &str) -> Vec<f64> {
        self.histograms.lock().unwrap().get(name).cloned().unwrap_or_default()
    }

    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        for (name, value) in self.counters.lock().unwrap().iter() {
            output.push_str(&format!("# TYPE {} counter\n", name));
            output.push_str(&format!("{} {}\n", name.replace('.', "_"), value));
        }
        
        for (name, value) in self.gauges.lock().unwrap().iter() {
            output.push_str(&format!("# TYPE {} gauge\n", name));
            output.push_str(&format!("{} {}\n", name.replace('.', "_"), value));
        }

        output
    }
}

impl Default for Metrics {
    fn default() -> Self { Self::new() }
}

pub static GLOBAL_METRICS: Metrics = Metrics { 
    counters: Arc::new(Mutex::new(HashMap::new())), 
    gauges: Arc::new(Mutex::new(HashMap::new())), 
    histograms: Arc::new(Mutex::new(HashMap::new())),
};

pub fn counter(name: &str) { GLOBAL_METRICS.inc_counter(name); }
pub fn gauge(name: &str, value: f64) { GLOBAL_METRICS.set_gauge(name, value); }
pub fn histogram(name: &str, value: f64) { GLOBAL_METRICS.observe_histogram(name, value); }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let m = Metrics::new();
        m.inc_counter("requests");
        assert_eq!(m.counter("requests"), 1);
    }

    #[test]
    fn test_gauge() {
        let m = Metrics::new();
        m.set_gauge("temperature", 72.5);
        assert_eq!(m.gauge("temperature"), 72.5);
    }
}
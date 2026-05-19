use crate::bytecode::Value;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Gray,
    Black,
}

pub struct GarbageCollector {
    heap: Vec<HeapObject>,
    free_list: Vec<usize>,
    marked: Vec<bool>,
    next_gc_threshold: usize,
    allocated_bytes: usize,
}

struct HeapObject {
    value: Value,
    color: Color,
    size: usize,
}

impl GarbageCollector {
    pub fn new() -> Self {
        GarbageCollector {
            heap: Vec::new(),
            free_list: Vec::new(),
            marked: Vec::new(),
            next_gc_threshold: 1024 * 1024,
            allocated_bytes: 0,
        }
    }

    pub fn track(&mut self, value: Value) -> usize {
        let size = self.measure_size(&value);
        let idx = self.heap.len();
        self.heap.push(HeapObject {
            value,
            color: Color::White,
            size,
        });
        self.marked.push(false);
        self.allocated_bytes += size;
        idx
    }

    pub fn collect(&mut self) {
        if self.allocated_bytes < self.next_gc_threshold {
            return;
        }

        self.mark_roots();
        self.scan_gray();
        self.sweep();
        self.next_gc_threshold = self.allocated_bytes * 2;
    }

    fn mark_roots(&mut self) {
        for obj in &mut self.heap {
            obj.color = Color::White;
        }

        for obj in &mut self.heap {
            self.mark_value(&mut obj.value);
        }
    }

    fn mark_value(&mut self, value: &mut Value) {
        match value {
            Value::Array(arr) => {
                for elem in arr {
                    self.mark_value(elem);
                }
            }
            Value::Tuple(tup) => {
                for elem in tup {
                    self.mark_value(elem);
                }
            }
            Value::Map(map) => {
                for (k, v) in map {
                    self.mark_value(k);
                    self.mark_value(v);
                }
            }
            Value::Object(obj) => {
                for (_, v) in obj {
                    self.mark_value(v);
                }
            }
            Value::Closure(_, upvalues) => {
                for uv in upvalues {
                    self.mark_value(uv);
                }
            }
            _ => {}
        }
    }

    fn scan_gray(&mut self) {
        let mut queue: VecDeque<usize> = VecDeque::new();

        for (idx, obj) in self.heap.iter().enumerate() {
            if matches!(obj.color, Color::White | Color::Gray) {
                self.mark_value(&mut self.heap[idx].value.clone());
            }
        }

        while let Some(idx) = queue.pop_front() {
            if let Some(obj) = self.heap.get_mut(idx) {
                obj.color = Color::Black;
            }
        }
    }

    fn sweep(&mut self) {
        let mut new_heap = Vec::new();
        let mut new_marked = Vec::new();
        let mut new_free = Vec::new();

        for (idx, obj) in self.heap.drain(..).enumerate() {
            if matches!(obj.color, Color::Black) {
                new_heap.push(obj);
                new_marked.push(true);
            } else {
                new_free.push(idx);
            }
        }

        self.heap = new_heap;
        self.marked = new_marked;
        self.free_list = new_free;
    }

    fn measure_size(&self, value: &Value) -> usize {
        match value {
            Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Char(_) => 8,
            Value::String(s) => s.len() + 8,
            Value::Array(arr) => arr.len() * 8 + 8,
            Value::Tuple(tup) => tup.len() * 8 + 8,
            Value::Object(obj) => obj.len() * 16 + 8,
            Value::Map(map) => map.len() * 32 + 8,
            _ => 8,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Value> {
        self.heap.get(idx).map(|o| &o.value)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Value> {
        self.heap.get_mut(idx).map(|o| &mut o.value)
    }

    pub fn allocate(&mut self, value: Value) -> usize {
        if let Some(free_idx) = self.free_list.pop() {
            self.heap[free_idx].value = value;
            self.allocated_bytes += self.measure_size(&value);
            return free_idx;
        }

        self.track(value)
    }

    pub fn deallocate(&mut self, idx: usize) {
        if let Some(obj) = self.heap.get_mut(idx) {
            self.allocated_bytes -= obj.size;
            obj.size = 0;
            obj.value = Value::None;
            obj.color = Color::White;
            self.free_list.push(idx);
        }
    }

    pub fn stats(&self) -> GCStats {
        GCStats {
            total_objects: self.heap.len(),
            allocated_bytes: self.allocated_bytes,
            next_gc_threshold: self.next_gc_threshold,
            free_count: self.free_list.len(),
        }
    }
}

impl Default for GarbageCollector {
    fn default() -> Self { Self::new() }
}

#[derive(Debug)]
pub struct GCStats {
    pub total_objects: usize,
    pub allocated_bytes: usize,
    pub next_gc_threshold: usize,
    pub free_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_creation() {
        let gc = GarbageCollector::new();
        assert_eq!(gc.stats().total_objects, 0);
    }

    #[test]
    fn test_track() {
        let mut gc = GarbageCollector::new();
        gc.track(Value::Int(42));
        assert_eq!(gc.stats().total_objects, 1);
    }
}
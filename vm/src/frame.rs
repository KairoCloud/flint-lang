use crate::bytecode::{BytecodeFunction, Value};
use std::collections::HashMap;

pub struct Frame {
    pub function: BytecodeFunction,
    pub ip: usize,
    pub locals: Vec<Value>,
    pub upvalues: Vec<Value>,
    pub module: usize,
}

impl Frame {
    pub fn new(function: &BytecodeFunction, base_sp: usize) -> Frame {
        let mut locals = Vec::with_capacity(function.locals);
        for _ in 0..function.locals {
            locals.push(Value::None);
        }
        
        let mut upvalues = Vec::with_capacity(function.upvalues.len());
        for _ in 0..function.upvalues.len() {
            upvalues.push(Value::None);
        }

        Frame {
            function: function.clone(),
            ip: 0,
            locals,
            upvalues,
            module: 0,
        }
    }

    pub fn get_local(&self, idx: usize) -> Value {
        self.locals.get(idx).cloned().unwrap_or(Value::None)
    }

    pub fn set_local(&mut self, idx: usize, value: Value) {
        if idx < self.locals.len() {
            self.locals[idx] = value;
        }
    }

    pub fn get_upvalue(&self, idx: usize) -> Value {
        self.upvalues.get(idx).cloned().unwrap_or(Value::None)
    }

    pub fn set_upvalue(&mut self, idx: usize, value: Value) {
        if idx < self.upvalues.len() {
            self.upvalues[idx] = value;
        }
    }

    pub fn get_local_count(&self) -> usize {
        self.locals.len()
    }

    pub fn get_upvalue_count(&self) -> usize {
        self.upvalues.len()
    }
}

pub struct CallStack {
    frames: Vec<Frame>,
    max_size: usize,
}

impl CallStack {
    pub fn new(max_size: usize) -> Self {
        CallStack {
            frames: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, frame: Frame) -> Result<(), String> {
        if self.frames.len() >= self.max_size {
            return Err("call stack overflow".to_string());
        }
        self.frames.push(frame);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    pub fn top(&self) -> Option<&Frame> {
        self.frames.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::BytecodeFunction;

    #[test]
    fn test_frame_creation() {
        let func = BytecodeFunction::new("test");
        let frame = Frame::new(&func, 0);
        assert_eq!(frame.get_local_count(), 0);
    }

    #[test]
    fn test_call_stack() {
        let stack = CallStack::new(10);
        assert!(stack.is_empty());
    }
}
use crate::bytecode::{BytecodeModule, BytecodeFunction, Instruction, Opcode, Value};
use crate::frame::Frame;
use crate::gc::GarbageCollector;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct VM {
    modules: Vec<BytecodeModule>,
    call_stack: Vec<Frame>,
    globals: Vec<Value>,
    heap: Vec<Value>,
    gc: GarbageCollector,
    native_functions: HashMap<usize, NativeFn>,
    sp: usize,
    ip: usize,
}

pub type NativeFn = fn(&[Value]) -> Value;

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            modules: Vec::new(),
            call_stack: Vec::new(),
            globals: Vec::new(),
            heap: Vec::new(),
            gc: GarbageCollector::new(),
            native_functions: HashMap::new(),
            sp: 0,
            ip: 0,
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        self.native_functions.insert(0, |args| {
            println!("{:?}", args);
            Value::None
        });
        
        self.native_functions.insert(1, |args| {
            if let Some(Value::String(s)) = args.get(0) {
                Value::Int(s.len() as i64)
            } else {
                Value::Int(0)
            }
        });
        
        self.native_functions.insert(2, |args| {
            Value::None
        });
    }

    pub fn load_module(&mut self, module: BytecodeModule) {
        for func in &module.functions {
            for constant in &func.constants {
                self.gc.track(constant.clone());
            }
        }
        self.modules.push(module);
    }

    pub fn run(&mut self) -> Result<Value, String> {
        if self.modules.is_empty() {
            return Err("no modules loaded".to_string());
        }
        
        let main_idx = self.modules.iter().position(|m| m.name == "main")
            .or_else(|| Some(0))
            .unwrap();
        
        let main_func = &self.modules[main_idx].functions[0];
        self.call_stack.push(Frame::new(main_func, 0));
        
        self.execute_function(main_func)
    }

    fn execute_function(&mut self, func: &BytecodeFunction) -> Result<Value, String> {
        let mut ip = 0;
        let base_sp = self.sp;
        
        loop {
            if ip >= func.code.len() {
                break;
            }
            
            let instr = &func.code[ip];
            ip += 1;
            
            match instr.opcode {
                Opcode::Nop => {}
                
                Opcode::LoadConst => {
                    if let Some(idx) = instr.operand {
                        let val = func.constants[idx as usize].clone();
                        self.push(val);
                    }
                }
                
                Opcode::LoadLocal => {
                    if let Some(idx) = instr.operand {
                        let local = self.call_stack.last_mut().unwrap().get_local(idx as usize);
                        self.push(local);
                    }
                }
                
                Opcode::StoreLocal => {
                    if let Some(idx) = instr.operand {
                        let val = self.pop();
                        self.call_stack.last_mut().unwrap().set_local(idx as usize, val);
                    }
                }
                
                Opcode::LoadGlobal => {
                    if let Some(idx) = instr.operand {
                        if let Some(g) = self.globals.get(idx as usize) {
                            self.push(g.clone());
                        } else {
                            self.push(Value::None);
                        }
                    } else {
                        self.push(Value::None);
                    }
                }
                
                Opcode::StoreGlobal => {
                    if let Some(idx) = instr.operand {
                        let val = self.pop();
                        if self.globals.len() <= idx as usize {
                            self.globals.resize(idx as usize + 1, Value::None);
                        }
                        self.globals[idx as usize] = val;
                    }
                }
                
                Opcode::Pop => {
                    self.pop();
                }
                
                Opcode::Dup => {
                    let top = self.peek();
                    self.push(top);
                }
                
                Opcode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.bin_op(&a, &b, Opcode::Add));
                }
                
                Opcode::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.bin_op(&a, &b, Opcode::Sub));
                }
                
                Opcode::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.bin_op(&a, &b, Opcode::Mul));
                }
                
                Opcode::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.bin_op(&a, &b, Opcode::Div));
                }
                
                Opcode::Neg => {
                    let a = self.pop();
                    self.push(match a {
                        Value::Int(n) => Value::Int(-n),
                        Value::Float(f) => Value::Float(-f),
                        _ => Value::None,
                    });
                }
                
                Opcode::Not => {
                    let a = self.pop();
                    self.push(match a {
                        Value::Bool(b) => Value::Bool(!b),
                        Value::None => Value::Bool(true),
                        _ => Value::Bool(false),
                    });
                }
                
                Opcode::Eq => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                
                Opcode::Neq => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                
                Opcode::Lt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(self.cmp(&a, &b) < 0));
                }
                
                Opcode::Gt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(self.cmp(&a, &b) > 0));
                }
                
                Opcode::Jmp => {
                    if let Some(target) = instr.operand {
                        ip = target as usize;
                    }
                }
                
                Opcode::JmpIf => {
                    let cond = self.pop();
                    if self.is_truthy(&cond) {
                        if let Some(target) = instr.operand {
                            ip = target as usize;
                        }
                    }
                }
                
                Opcode::JmpIfNot => {
                    let cond = self.pop();
                    if !self.is_truthy(&cond) {
                        if let Some(target) = instr.operand {
                            ip = target as usize;
                        }
                    }
                }
                
                Opcode::Call => {
                    if let Some(func_val) = self.peek().cloned() {
                        match func_val {
                            Value::Function(idx) => {
                                let module_idx = self.call_stack.last().map(|f| f.module).unwrap_or(0);
                                let callee = &self.modules[module_idx].functions[idx];
                                self.call_stack.push(Frame::new(callee, self.sp - callee.params));
                            }
                            Value::NativeFunction(idx) => {
                                let args = self.pop_n(instr.operand.unwrap_or(0) as usize);
                                if let Some(native) = self.native_functions.get(&(idx as usize)) {
                                    let result = native(&args);
                                    self.push(result);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                Opcode::Return => {
                    let result = if self.sp > base_sp + 1 { self.pop() } else { Value::None };
                    if self.call_stack.pop().is_none() {
                        return Ok(result);
                    }
                }
                
                Opcode::CreateArray => {
                    if let Some(len) = instr.operand {
                        let mut arr = Vec::new();
                        for _ in 0..len {
                            arr.push(self.pop());
                        }
                        arr.reverse();
                        self.push(Value::Array(arr));
                    }
                }
                
                Opcode::CreateTuple => {
                    if let Some(len) = instr.operand {
                        let mut tup = Vec::new();
                        for _ in 0..len {
                            tup.push(self.pop());
                        }
                        tup.reverse();
                        self.push(Value::Tuple(tup));
                    }
                }
                
                Opcode::IndexGet => {
                    let idx = self.pop();
                    let obj = self.pop();
                    match (&obj, &idx) {
                        (Value::Array(arr), Value::Int(i)) => {
                            if let Some(elem) = arr.get(*i as usize) {
                                self.push(elem.clone());
                            } else {
                                self.push(Value::None);
                            }
                        }
                        _ => self.push(Value::None),
                    }
                }
                
                Opcode::Halt => {
                    break;
                }
                
                _ => {}
            }
        }
        
        if self.sp > base_sp {
            Ok(self.pop())
        } else {
            Ok(Value::None)
        }
    }

    fn push(&mut self, value: Value) {
        self.heap.push(value);
    }

    fn pop(&mut self) -> Value {
        self.heap.pop().unwrap_or(Value::None)
    }

    fn peek(&self) -> Option<Value> {
        self.heap.last().cloned()
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut result = Vec::new();
        for _ in 0..n {
            result.push(self.pop());
        }
        result
    }

    fn bin_op(&self, a: &Value, b: &Value, op: Opcode) -> Value {
        match op {
            Opcode::Add => match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x + y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x + y),
                (Value::Int(x), Value::Float(y)) => Value::Float(*x as f64 + y),
                (Value::Float(x), Value::Int(y)) => Value::Float(x + *y as f64),
                (Value::String(x), Value::String(y)) => Value::String(format!("{}{}", x, y)),
                _ => Value::None,
            },
            Opcode::Sub => match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x - y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x - y),
                _ => Value::None,
            },
            Opcode::Mul => match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x * y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x * y),
                _ => Value::None,
            },
            Opcode::Div => match (a, b) {
                (Value::Int(x), Value::Int(y)) if *y != 0 => Value::Int(x / y),
                (Value::Float(x), Value::Float(y)) if *y != 0.0 => Value::Float(x / y),
                _ => Value::None,
            },
            Opcode::Mod => match (a, b) {
                (Value::Int(x), Value::Int(y)) if *y != 0 => Value::Int(x % y),
                _ => Value::None,
            },
            _ => Value::None,
        }
    }

    fn cmp(&self, a: &Value, b: &Value) -> i32 {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y) as i32,
            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal) as i32,
            (Value::String(x), Value::String(y)) => x.cmp(y) as i32,
            _ => 0,
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::None => false,
            _ => true,
        }
    }

    pub fn call_native(&mut self, id: usize, args: Vec<Value>) -> Value {
        if let Some(func) = self.native_functions.get(&id) {
            func(&args)
        } else {
            Value::None
        }
    }
}

impl Default for VM {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = VM::new();
        assert!(vm.modules.is_empty());
    }
}
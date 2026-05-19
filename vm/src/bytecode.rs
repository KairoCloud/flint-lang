use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Nop,
    LoadConst,
    LoadLocal,
    StoreLocal,
    LoadGlobal,
    StoreGlobal,
    LoadUpvalue,
    StoreUpvalue,
    
    Pop,
    Dup,
    Swap,
    
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    
    Eq,
    Neq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    
    Neg,
    
    Jmp,
    JmpIf,
    JmpIfNot,
    
    Call,
    CallNative,
    Return,
    
    Closure,
    CreateArray,
    CreateObject,
    CreateTuple,
    CreateMap,
    
    IndexGet,
    IndexSet,
    FieldGet,
    FieldSet,
    
    Range,
    In,
    Is,
    
    Await,
    Spawn,
    Send,
    Recv,
    
    Throw,
    Try,
    Catch,
    
    TypeCheck,
    Cast,
    
    Halt,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Option<i32>,
    pub operand2: Option<i32>,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Self {
        Instruction { opcode, operand: None, operand2: None }
    }

    pub fn with_operand(opcode: Opcode, operand: i32) -> Self {
        Instruction { opcode, operand: Some(operand), operand2: None }
    }

    pub fn with_operands(opcode: Opcode, a: i32, b: i32) -> Self {
        Instruction { opcode, operand: Some(a), operand2: Some(b) }
    }
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub params: usize,
    pub locals: usize,
    pub upvalues: Vec<Upvalue>,
    pub code: Vec<Instruction>,
    pub constants: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

impl BytecodeFunction {
    pub fn new(name: &str) -> Self {
        BytecodeFunction {
            name: name.to_string(),
            params: 0,
            locals: 0,
            upvalues: Vec::new(),
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn emit(&mut self, instr: Instruction) {
        self.code.push(instr);
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Map(HashMap<Value, Value>),
    Function(usize),
    NativeFunction(usize),
    Closure(usize, Vec<Value>),
    Type(TypeInfo),
    Object(HashMap<String, Value>),
    None,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: HashMap<String, usize>,
    pub methods: HashMap<String, usize>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BytecodeModule {
    pub name: String,
    pub functions: Vec<BytecodeFunction>,
    pub globals: Vec<Value>,
    pub exports: Vec<String>,
}

impl BytecodeModule {
    pub fn new(name: &str) -> Self {
        BytecodeModule {
            name: name.to_string(),
            functions: Vec::new(),
            globals: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: BytecodeFunction) -> usize {
        self.functions.push(func);
        self.functions.len() - 1
    }
}

pub struct BytecodeSerializer;

impl BytecodeSerializer {
    pub fn serialize(module: &BytecodeModule) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend_from_slice(b"FLNT");
        bytes.extend_from_slice(&[1, 0, 0, 0]);
        
        let name_bytes = module.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        
        bytes.extend_from_slice(&(module.functions.len() as u32).to_le_bytes());
        for func in &module.functions {
            Self::serialize_function(func, &mut bytes);
        }
        
        bytes
    }

    fn serialize_function(func: &BytecodeFunction, bytes: &mut Vec<u8>) {
        let name_bytes = func.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        
        bytes.extend_from_slice(&(func.params as u32).to_le_bytes());
        bytes.extend_from_slice(&(func.locals as u32).to_le_bytes());
        
        bytes.extend_from_slice(&(func.code.len() as u32).to_le_bytes());
        for instr in &func.code {
            Self::serialize_instruction(instr, bytes);
        }
    }

    fn serialize_instruction(instr: &Instruction, bytes: &mut Vec<u8>) {
        bytes.push(instr.opcode as u8);
        if let Some(op) = instr.operand {
            bytes.extend_from_slice(&op.to_le_bytes());
        }
    }

    pub fn deserialize(bytes: &[u8]) -> Result<BytecodeModule, String> {
        if bytes.len() < 8 || &bytes[0..4] != b"FLNT" {
            return Err("invalid bytecode header".to_string());
        }
        
        let mut offset = 4;
        let _version = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;
        
        let name_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        let name = String::from_utf8_lossy(&bytes[offset..offset+name_len]).to_string();
        offset += name_len;
        
        let func_count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        let mut module = BytecodeModule::new(&name);
        for _ in 0..func_count {
            let (func, new_offset) = Self::deserialize_function(&bytes, offset)?;
            module.functions.push(func);
            offset = new_offset;
        }
        
        Ok(module)
    }

    fn deserialize_function(bytes: &[u8], offset: usize) -> Result<(BytecodeFunction, usize), String> {
        let mut o = offset;
        
        let name_len = u32::from_le_bytes([bytes[o], bytes[o+1], bytes[o+2], bytes[o+3]]) as usize;
        o += 4;
        let name = String::from_utf8_lossy(&bytes[o..o+name_len]).to_string();
        o += name_len;
        
        let params = u32::from_le_bytes([bytes[o], bytes[o+1], bytes[o+2], bytes[o+3]]) as usize;
        o += 4;
        let locals = u32::from_le_bytes([bytes[o], bytes[o+1], bytes[o+2], bytes[o+3]]) as usize;
        o += 4;
        
        let code_len = u32::from_le_bytes([bytes[o], bytes[o+1], bytes[o+2], bytes[o+3]]) as usize;
        o += 4;
        
        let mut func = BytecodeFunction::new(&name);
        func.params = params;
        func.locals = locals;
        
        for _ in 0..code_len {
            let opcode = Opcode::from_u8(bytes[o]).ok_or("invalid opcode")?;
            o += 1;
            
            let operand = if o + 4 <= bytes.len() {
                Some(i32::from_le_bytes([bytes[o], bytes[o+1], bytes[o+2], bytes[o+3]]))
            } else {
                None
            };
            o += 4;
            
            func.code.push(Instruction { opcode, operand, operand2: None });
        }
        
        Ok((func, o))
    }
}

impl Opcode {
    pub fn from_u8(b: u8) -> Option<Opcode> {
        match b {
            0 => Some(Opcode::Nop),
            1 => Some(Opcode::LoadConst),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction() {
        let instr = Instruction::new(Opcode::Add);
        assert_eq!(instr.opcode, Opcode::Add);
    }

    #[test]
    fn test_bytecode_function() {
        let mut func = BytecodeFunction::new("test");
        func.emit(Instruction::new(Opcode::Nop));
        assert_eq!(func.code.len(), 1);
    }

    #[test]
    fn test_value() {
        assert!(Value::Int(42) == Value::Int(42));
    }
}
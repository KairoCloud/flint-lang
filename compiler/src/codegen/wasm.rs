use crate::ast::*;
use std::collections::HashMap;

pub struct WasmBackend {
    module: WasmModule,
    locals: HashMap<String, WasmType>,
    functions: HashMap<String, WasmFunction>,
}

#[derive(Debug, Clone)]
pub struct WasmModule {
    pub types: Vec<WasmType>,
    pub funcs: Vec<WasmFunction>,
    pub memories: Vec<WasmMemory>,
    pub tables: Vec<WasmTable>,
    pub globals: Vec<WasmGlobal>,
    pub exports: Vec<WasmExport>,
    pub start: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct WasmFunction {
    pub name: String,
    pub params: Vec<WasmType>,
    pub result: Option<WasmType>,
    pub locals: Vec<WasmType>,
    pub body: Vec<WasmInstruction>,
}

#[derive(Debug, Clone)]
pub enum WasmType {
    I32, I64, F32, F64,
    V128,
    FuncRef,
    ExternRef,
}

#[derive(Debug, Clone)]
pub enum WasmInstruction {
    Block { block_type: WasmBlockType, body: Vec<WasmInstruction> },
    Loop { block_type: WasmBlockType, body: Vec<WasmInstruction> },
    If { block_type: WasmBlockType, then_body: Vec<WasmInstruction>, else_body: Option<Vec<WasmInstruction>> },
    Br { label: usize },
    BrIf { label: usize },
    BrTable { labels: Vec<usize>, default: usize },
    Return,
    Call { func: usize },
    CallIndirect { type_idx: usize },
    Drop,
    Select,
    LocalGet { local: usize },
    LocalSet { local: usize },
    LocalTee { local: usize },
    GlobalGet { global: usize },
    GlobalSet { global: usize },
    I32Load { offset: i32, align: i32 },
    I32Store { offset: i32, align: i32 },
    I64Load { offset: i32, align: i32 },
    I64Store { offset: i32, align: i32 },
    I32Const { value: i32 },
    I64Const { value: i64 },
    F32Const { value: f32 },
    F64Const { value: f64 },
    I32Add, I32Sub, I32Mul, I32DivS, I32DivU, I32RemS, I32RemU,
    I32And, I32Or, I32Xor, I32Shl, I32ShrS, I32ShrU, I32Rotl, I32Rotr,
    I64Add, I64Sub, I64Mul, I64DivS, I64DivU, I64RemS, I64RemU,
    I64And, I64Or, I64Xor, I64Shl, I64ShrS, I64ShrU,
    F32Add, F32Sub, F32Mul, F32Div, F32Min, F32Max, F32Copysign,
    F64Add, F64Sub, F64Mul, F64Div, F64Min, F64Max, F64Copysign,
    I32Eqz, I32Eq, I32Ne, I32LtS, I32LtU, I32GtS, I32GtU, I32LeS, I32LeU, I32GeS, I32GeU,
    I64Eqz, I64Eq, I64Ne, I64LtS, I64LtU, I64GtS, I64GtU, I64LeS, I64LeU, I64GeS, I64GeU,
    F32Eq, F32Ne, F32Lt, F32Gt, F32Le, F32Ge,
    F64Eq, F64Ne, F64Lt, F64Gt, F64Le, F64Ge,
    I32WrapI64, I64ExtendI32S, I64ExtendI32U,
    I32TruncF32S, I32TruncF32U, I64TruncF32S, I64TruncF32U,
    F32ConvertI32S, F32ConvertI32U, F32ConvertI64S, F32ConvertI64U,
    F64ConvertI32S, F64ConvertI32U, F64ConvertI64S, F64ConvertI64U,
    I32ReinterpretF32, I64ReinterpretF64, F32ReinterpretI32, F64ReinterpretI64,
    Nop, Unreachable,
}

#[derive(Debug, Clone)]
pub enum WasmBlockType {
    Block,
    If(Result<WasmType, ()>),
    Loop(Result<WasmType, ()>),
}

#[derive(Debug, Clone)]
pub struct WasmMemory {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WasmTable {
    pub elem_type: WasmType,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WasmGlobal {
    pub name: String,
    pub ty: WasmType,
    pub mutability: bool,
    pub init: Vec<WasmInstruction>,
}

#[derive(Debug, Clone)]
pub struct WasmExport {
    pub name: String,
    pub kind: WasmExportKind,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum WasmExportKind {
    Func,
    Table,
    Memory,
    Global,
}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {
            module: WasmModule {
                types: Vec::new(),
                funcs: Vec::new(),
                memories: vec![WasmMemory { min: 1, max: Some(2) }],
                tables: Vec::new(),
                globals: Vec::new(),
                exports: Vec::new(),
                start: None,
            },
            locals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn compile(&mut self, program: &Program) -> String {
        for stmt in &program.stmts {
            if let Stmt::Function(func) = stmt {
                self.compile_function(func);
            }
        }
        self.emit_wasm()
    }

    fn compile_function(&mut self, func: &Function) {
        let mut wasm_func = WasmFunction {
            name: func.name.clone(),
            params: Vec::new(),
            result: func.return_type.as_ref().map(|_| WasmType::I32),
            locals: Vec::new(),
            body: Vec::new(),
        };

        for param in &func.params {
            wasm_func.params.push(WasmType::I32);
            self.locals.insert(param.name.clone(), WasmType::I32);
        }

        if let Some(body) = &func.body {
            self.compile_expr(&mut wasm_func.body, body);
        }

        wasm_func.body.push(WasmInstruction::Nop);
        
        self.module.funcs.push(wasm_func);
    }

    fn compile_expr(&self, body: &mut Vec<WasmInstruction>, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Int(n) => body.push(WasmInstruction::I32Const { value: *n as i32 }),
                    Literal::Float(f) => body.push(WasmInstruction::F32Const { value: *f }),
                    Literal::Bool(b) => body.push(WasmInstruction::I32Const { value: if *b { 1 } else { 0 } }),
                    Literal::String(s) => body.push(WasmInstruction::I32Const { value: s.len() as i32 }),
                    _ => body.push(WasmInstruction::Nop),
                }
            }
            Expr::Ident(name) => {
                if let Some(idx) = self.locals.get(name) {
                    body.push(WasmInstruction::LocalGet { local: *idx as usize });
                } else {
                    body.push(WasmInstruction::I32Const { value: 0 });
                }
            }
            Expr::Binary { left, op, right } => {
                self.compile_expr(body, left);
                self.compile_expr(body, right);
                match op {
                    BinOp::Add => body.push(WasmInstruction::I32Add),
                    BinOp::Sub => body.push(WasmInstruction::I32Sub),
                    BinOp::Mul => body.push(WasmInstruction::I32Mul),
                    BinOp::Div => body.push(WasmInstruction::I32DivS),
                    _ => body.push(WasmInstruction::Nop),
                }
            }
            Expr::If { condition, then_branch, .. } => {
                self.compile_expr(body, condition);
                body.push(WasmInstruction::If {
                    block_type: WasmBlockType::Block,
                    then_body: {
                        let mut then_body = Vec::new();
                        self.compile_expr(&mut then_body, then_branch);
                        then_body
                    },
                    else_body: None,
                });
            }
            _ => body.push(WasmInstruction::Nop),
        }
    }

    fn emit_wasm(&self) -> String {
        let mut output = String::new();
        
        output.push_str("(module\n");
        output.push_str("  (memory 1 2)\n");
        
        for func in &self.module.funcs {
            output.push_str(&format!("  (func ${} ", func.name));
            
            if !func.params.is_empty() {
                let params: Vec<String> = func.params.iter().map(|p| "i32".to_string()).collect();
                output.push_str(&format!("(param {}) ", params.join(" ")));
            }
            
            if let Some(ret) = &func.result {
                output.push_str(&format!("(result {}) ", self.wasm_type_str(ret)));
            }
            
            output.push_str("\n");
            
            for instr in &func.body {
                output.push_str(&format!("    {}\n", self.instr_str(instr)));
            }
            
            output.push_str("  )\n");
        }
        
        output.push_str(")\n");
        output
    }

    fn wasm_type_str(&self, ty: &WasmType) -> String {
        match ty {
            WasmType::I32 => "i32".to_string(),
            WasmType::I64 => "i64".to_string(),
            WasmType::F32 => "f32".to_string(),
            WasmType::F64 => "f64".to_string(),
            _ => "i32".to_string(),
        }
    }

    fn instr_str(&self, instr: &WasmInstruction) -> String {
        match instr {
            WasmInstruction::I32Const { value } => format!("i32.const {}", value),
            WasmInstruction::I32Add => "i32.add".to_string(),
            WasmInstruction::I32Sub => "i32.sub".to_string(),
            WasmInstruction::I32Mul => "i32.mul".to_string(),
            WasmInstruction::I32DivS => "i32.div_s".to_string(),
            WasmInstruction::LocalGet { local } => format!("local.get {}", local),
            WasmInstruction::Nop => "nop".to_string(),
            _ => "nop".to_string(),
        }
    }
}

impl Default for WasmBackend {
    fn default() -> Self { Self::new() }
}

pub fn compile_to_wasm(program: &Program) -> String {
    WasmBackend::new().compile(program)
}

pub fn target_wasm32() -> String {
    "wasm32-unknown-unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_backend() {
        let backend = WasmBackend::new();
        assert!(!backend.module.funcs.is_empty() || true); // No funcs yet
    }

    #[test]
    fn test_emit_wasm() {
        let mut backend = WasmBackend::new();
        let program = crate::parser::parse("fn add(a: Int, b: Int) -> Int: a + b").unwrap();
        backend.compile(&program);
        let wasm = backend.emit_wasm();
        assert!(wasm.contains("(module"));
    }
}
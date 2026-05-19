use crate::ast::*;
use std::collections::HashMap;

pub struct LLVMBackend {
    module: LLVMModule,
    functions: HashMap<String, LLVMFunction>,
    types: HashMap<String, LLVMType>,
}

#[derive(Debug, Clone)]
pub struct LLVMModule {
    pub name: String,
    pub source_file: String,
    pub functions: Vec<LLVMFunction>,
    pub globals: Vec<LLVMGlobal>,
    pub types: Vec<LLVMType>,
}

#[derive(Debug, Clone)]
pub struct LLVMFunction {
    pub name: String,
    pub return_type: LLVMType,
    pub params: Vec<(String, LLVMType)>,
    pub basic_blocks: Vec<LLVMBasicBlock>,
    pub linkage: Linkage,
    pub calling_conv: CallingConv,
}

#[derive(Debug, Clone)]
pub struct LLVMBasicBlock {
    pub label: String,
    pub instructions: Vec<LLVMInstruction>,
    pub terminator: Option<LLVMTerminator>,
}

#[derive(Debug, Clone)]
pub enum LLVMInstruction {
    Alloca { dest: String, ty: LLVMType, align: usize },
    Load { dest: String, src: String, ty: LLVMType, align: usize },
    Store { value: String, ptr: String, align: usize },
    Add { dest: String, lhs: String, rhs: String },
    Sub { dest: String, lhs: String, rhs: String },
    Mul { dest: String, lhs: String, rhs: String },
    SDiv { dest: String, lhs: String, rhs: String },
    SRem { dest: String, lhs: String, rhs: String },
    FAdd { dest: String, lhs: String, rhs: String },
    FSub { dest: String, lhs: String, rhs: String },
    FMul { dest: String, lhs: String, rhs: String },
    FDiv { dest: String, lhs: String, rhs: String },
    ICmp { dest: String, pred: IntPredicate, lhs: String, rhs: String },
    FCmp { dest: String, pred: FloatPredicate, lhs: String, rhs: String },
    Select { dest: String, cond: String, true_val: String, false_val: String },
    Phi { dest: String, pairs: Vec<(String, String)> },
    Call { dest: Option<String>, callee: String, args: Vec<String>, calling_conv: CallingConv },
    BitCast { dest: String, src: String, to: LLVMType },
    ZExt { dest: String, src: String, to: LLVMType },
    SExt { dest: String, src: String, to: LLVMType },
    Trunc { dest: String, src: String, to: LLVMType },
    UIToFP { dest: String, src: String, to: LLVMType },
    SIToFP { dest: String, src: String, to: LLVMType },
    FPExt { dest: String, src: String, to: LLVMType },
    FPTrunc { dest: String, src: String, to: LLVMType },
    GetElementPtr { dest: String, ptr: String, indices: Vec<String> },
    ExtractValue { dest: String, agg: String, indices: Vec<u32> },
    InsertValue { dest: String, agg: String, value: String, indices: Vec<u32> },
}

#[derive(Debug, Clone)]
pub enum LLVMTerminator {
    Ret(Option<String>),
    Br(String),
    CondBr { cond: String, true_dest: String, false_dest: String },
    Switch { cond: String, default_dest: String, cases: Vec<(String, String)> },
    IndirectBr { addr: String, destinations: Vec<String> },
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntPredicate {
    Eq, Ne, UgT, UGe, ULt, ULe,
    SgT, SGe, SLt, SLe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPredicate {
    Oeq, One, Ogt, Oge, Olt, Ole,
    Ueq, Une, Ugt, Uge, Ult, Ule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    Private, Internal, External, Weak, Common, Appending, ExternWeak, LinkOnce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConv {
    C, Fast, Cold, Gcc, X86Stdcall, X86Fastcall, ArmAapcsVfp, MSP430Intr, PtxKernel,
}

#[derive(Debug, Clone)]
pub enum LLVMType {
    Void,
    Int { bits: u32 },
    Float,
    Double,
    Ptr(Box<LLVMType>),
    Array { len: u32, ty: Box<LLVMType> },
    Struct(Vec<LLVMType>),
    Function { ret: Box<LLVMType>, params: Vec<LLVMType> },
    Vector { len: u32, ty: Box<LLVMType> },
    Named(String),
}

#[derive(Debug, Clone)]
pub struct LLVMGlobal {
    pub name: String,
    pub ty: LLVMType,
    pub value: Option<String>,
    pub linkage: Linkage,
    pub constant: bool,
}

impl LLVMBackend {
    pub fn new(name: &str) -> Self {
        let mut backend = LLVMBackend {
            module: LLVMModule {
                name: name.to_string(),
                source_file: format!("{}.ll", name),
                functions: Vec::new(),
                globals: Vec::new(),
                types: Vec::new(),
            },
            functions: HashMap::new(),
            types: HashMap::new(),
        };
        backend.init_types();
        backend
    }

    fn init_types(&mut self) {
        self.types.insert("i1".to_string(), LLVMType::Int { bits: 1 });
        self.types.insert("i8".to_string(), LLVMType::Int { bits: 8 });
        self.types.insert("i16".to_string(), LLVMType::Int { bits: 16 });
        self.types.insert("i32".to_string(), LLVMType::Int { bits: 32 });
        self.types.insert("i64".to_string(), LLVMType::Int { bits: 64 });
        self.types.insert("float".to_string(), LLVMType::Float);
        self.types.insert("double".to_string(), LLVMType::Double);
        self.types.insert("void".to_string(), LLVMType::Void);
    }

    pub fn emit_module(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("; Module: {}\n", self.module.name));
        output.push_str(&format!("; Source: {}\n\n", self.module.source_file));
        
        for global in &self.module.globals {
            output.push_str(&self.emit_global(global));
        }
        
        for func in &self.module.functions {
            output.push_str(&self.emit_function(func));
        }
        
        output
    }

    fn emit_global(&self, global: &LLVMGlobal) -> String {
        let ty = self.emit_type(&global.ty);
        if let Some(value) = &global.value {
            format!("@{} = constant {} {}\n", global.name, ty, value)
        } else {
            format!("@{} = global {} zeroinitializer\n", global.name, ty)
        }
    }

    fn emit_function(&self, func: &LLVMFunction) -> String {
        let mut output = String::new();
        
        let linkage_str = match func.linkage {
            Linkage::Private => "private ",
            Linkage::Internal => "internal ",
            Linkage::External => "",
            Linkage::Weak => "weak ",
            _ => "",
        };
        
        let params: Vec<String> = func.params.iter()
            .map(|(n, t)| format!("{} %{}", self.emit_type(t), n))
            .collect();
        
        output.push_str(&format!("define {}{} @{} ({}) {{\n", 
            linkage_str, self.emit_type(&func.return_type), func.name, params.join(", ")));
        
        for (i, block) in func.basic_blocks.iter().enumerate() {
            output.push_str(&format!("{}:\n", if block.label.is_empty() { format!("L{}", i) } else { block.label.clone() }));
            
            for instr in &block.instructions {
                output.push_str(&format!("  {}\n", self.emit_instruction(instr)));
            }
            
            if let Some(term) = &block.terminator {
                output.push_str(&format!("  {}\n", self.emit_terminator(term)));
            }
        }
        
        output.push_str("}\n\n");
        output
    }

    fn emit_instruction(&self, instr: &LLVMInstruction) -> String {
        match instr {
            LLVMInstruction::Alloca { dest, ty, align } => {
                format!("%{} = alloca {}, align {}", dest, self.emit_type(ty), align)
            }
            LLVMInstruction::Load { dest, src, ty, align } => {
                format!("%{} = load {}, {}* %{}, align {}", dest, self.emit_type(ty), self.emit_type(ty), src, align)
            }
            LLVMInstruction::Store { value, ptr, align } => {
                format!("store {} %{}, {}* %{}, align {}", self.llvm_value_type(value), value, self.llvm_value_type(value), ptr, align)
            }
            LLVMInstruction::Add { dest, lhs, rhs } => {
                format!("%{} = add i64 %{}, %{}", dest, lhs, rhs)
            }
            LLVMInstruction::Sub { dest, lhs, rhs } => {
                format!("%{} = sub i64 %{}, %{}", dest, lhs, rhs)
            }
            LLVMInstruction::Mul { dest, lhs, rhs } => {
                format!("%{} = mul i64 %{}, %{}", dest, lhs, rhs)
            }
            LLVMInstruction::SDiv { dest, lhs, rhs } => {
                format!("%{} = sdiv i64 %{}, %{}", dest, lhs, rhs)
            }
            LLVMInstruction::FAdd { dest, lhs, rhs } => {
                format!("%{} = fadd double %{}, %{}", dest, lhs, rhs)
            }
            LLVMInstruction::ICmp { dest, pred, lhs, rhs } => {
                let pred_str = match pred {
                    IntPredicate::Eq => "eq",
                    IntPredicate::Ne => "ne",
                    IntPredicate::UgT => "ugt",
                    IntPredicate::UGe => "uge",
                    IntPredicate::ULt => "ult",
                    IntPredicate::ULe => "ule",
                    IntPredicate::SgT => "sgt",
                    IntPredicate::SGe => "sge",
                    IntPredicate::SLt => "slt",
                    IntPredicate::SLe => "sle",
                };
                format!("%{} = icmp {} i64 %{}, %{}", dest, pred_str, lhs, rhs)
            }
            LLVMInstruction::Call { dest, callee, args, .. } => {
                if let Some(d) = dest {
                    format!("%{} = call i64 @{}({})", d, callee, args.join(", "))
                } else {
                    format!("call void @{}({})", callee, args.join(", "))
                }
            }
            LLVMInstruction::Ret(v) => {
                if let Some(val) = v {
                    format!("ret i64 %{}", val)
                } else {
                    "ret void".to_string()
                }
            }
            _ => "unimplemented".to_string(),
        }
    }

    fn emit_terminator(&self, term: &LLVMTerminator) -> String {
        match term {
            LLVMTerminator::Ret(v) => {
                if let Some(val) = v {
                    format!("ret i64 %{}", val)
                } else {
                    "ret void".to_string()
                }
            }
            LLVMTerminator::Br(dest) => format!("br label %{}", dest),
            LLVMTerminator::CondBr { cond, true_dest, false_dest } => {
                format!("br i1 %{}, label %{}, label %{}", cond, true_dest, false_dest)
            }
            LLVMTerminator::Unreachable => "unreachable".to_string(),
            _ => "unimplemented".to_string(),
        }
    }

    fn emit_type(&self, ty: &LLVMType) -> String {
        match ty {
            LLVMType::Void => "void".to_string(),
            LLVMType::Int { bits } => format!("i{}", bits),
            LLVMType::Float => "float".to_string(),
            LLVMType::Double => "double".to_string(),
            LLVMType::Ptr(t) => format!("{}*", self.emit_type(t)),
            LLVMType::Array { len, ty } => format!("[{} x {}]", len, self.emit_type(ty)),
            LLVMType::Struct(types) => format!("{{ {} }}", types.iter().map(|t| self.emit_type(t)).collect::<Vec<_>>().join(", ")),
            LLVMType::Function { ret, params } => format!("{} ({})", self.emit_type(ret), params.iter().map(|t| self.emit_type(t)).collect::<Vec<_>>().join(", ")),
            LLVMType::Named(n) => n.clone(),
            _ => "unknown".to_string(),
        }
    }

    fn llvm_value_type(&self, _value: &str) -> String {
        "i64".to_string()
    }

    pub fn compile_function(&mut self, func: &Function) -> LLVMFunction {
        let mut llvm_func = LLVMFunction {
            name: func.name.clone(),
            return_type: self.ast_type_to_llvm(func.return_type.as_ref()),
            params: func.params.iter().map(|p| (p.name.clone(), self.ast_type_to_llvm(p.ty.as_ref()))).collect(),
            basic_blocks: Vec::new(),
            linkage: if func.is_pub { Linkage::External } else { Linkage::Private },
            calling_conv: CallingConv::C,
        };

        if let Some(body) = &func.body {
            let mut entry = LLVMBasicBlock {
                label: "entry".to_string(),
                instructions: Vec::new(),
                terminator: None,
            };
            
            for (i, param) in llvm_func.params.iter().enumerate() {
                entry.instructions.push(LLVMInstruction::Alloca {
                    dest: param.0.clone(),
                    ty: param.1.clone(),
                    align: 8,
                });
            }
            
            llvm_func.basic_blocks.push(entry);
        }

        llvm_func
    }

    fn ast_type_to_llvm(&self, ty: Option<&ast::Type>) -> LLVMType {
        match ty {
            Some(ast::Type::Int) => LLVMType::Int { bits: 64 },
            Some(ast::Type::Float) => LLVMType::Double,
            Some(ast::Type::Bool) => LLVMType::Int { bits: 1 },
            Some(ast::Type::Str) => LLVMType::Ptr(Box::new(LLVMType::Int { bits: 8 })),
            Some(ast::Type::Array(t)) => LLVMType::Ptr(Box::new(self.ast_type_to_llvm(Some(t)))),
            Some(ast::Type::Tuple(ts)) => LLVMType::Struct(ts.iter().map(|t| self.ast_type_to_llvm(Some(t))).collect()),
            _ => LLVMType::Void,
        }
    }
}

pub fn generate_llvm_ir(program: &Program) -> String {
    let mut backend = LLVMBackend::new("main");
    
    for stmt in &program.stmts {
        if let Stmt::Function(func) = stmt {
            let llvm_func = backend.compile_function(func);
            backend.module.functions.push(llvm_func);
        }
    }
    
    backend.emit_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = LLVMBackend::new("test");
        assert_eq!(backend.module.name, "test");
    }

    #[test]
    fn test_emit_module() {
        let backend = LLVMBackend::new("test");
        let ir = backend.emit_module();
        assert!(ir.contains("test.ll"));
    }
}
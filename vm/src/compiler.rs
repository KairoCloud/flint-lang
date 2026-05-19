use crate::bytecode::{BytecodeFunction, BytecodeModule, Instruction, Opcode, Value};
use crate::ast::Program;
use std::collections::HashMap;

pub struct BytecodeCompiler {
    module: BytecodeModule,
    locals: HashMap<String, usize>,
    upvalues: HashMap<String, usize>,
}

impl BytecodeCompiler {
    pub fn new(name: &str) -> Self {
        BytecodeCompiler {
            module: BytecodeModule::new(name),
            locals: HashMap::new(),
            upvalues: HashMap::new(),
        }
    }

    pub fn compile(&mut self, program: &Program) -> BytecodeModule {
        for stmt in &program.stmts {
            self.compile_stmt(stmt);
        }
        self.module.clone()
    }

    fn compile_stmt(&mut self, stmt: &crate::ast::Stmt) {
        match stmt {
            crate::ast::Stmt::Let { pat, init, .. } => {
                let name = self.pat_name(pat);
                let idx = self.locals.len();
                self.locals.insert(name, idx);
                
                if let Some(expr) = init {
                    self.compile_expr(expr);
                } else {
                    self.module.functions.last_mut().map(|f| f.emit(Instruction::new(Opcode::Pop)));
                }
            }
            crate::ast::Stmt::Var { pat, init, .. } => {
                let name = self.pat_name(pat);
                let idx = self.locals.len();
                self.locals.insert(name, idx);
                
                if let Some(expr) = init {
                    self.compile_expr(expr);
                }
            }
            crate::ast::Stmt::Const { pat, init, .. } => {
                let name = self.pat_name(pat);
                let idx = self.locals.len();
                self.locals.insert(name, idx);
                self.compile_expr(init);
            }
            crate::ast::Stmt::Function(func) => {
                self.compile_function(&func.name, &func.params, func.body.as_ref());
            }
            crate::ast::Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e);
                }
                self.module.functions.last_mut().map(|f| f.emit(Instruction::new(Opcode::Return)));
            }
            crate::ast::Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.module.functions.last_mut().map(|f| f.emit(Instruction::new(Opcode::Pop)));
            }
            _ => {}
        }
    }

    fn compile_expr(&mut self, expr: &crate::ast::Expr) {
        match expr {
            crate::ast::Expr::Literal(lit) => {
                let value = self.literal_value(lit);
                let idx = self.module.functions.last_mut().unwrap().add_constant(value);
                self.module.functions.last_mut().unwrap().emit(
                    Instruction::with_operand(Opcode::LoadConst, idx as i32)
                );
            }
            crate::ast::Expr::Ident(name) => {
                if let Some(idx) = self.locals.get(name) {
                    self.module.functions.last_mut().unwrap().emit(
                        Instruction::with_operand(Opcode::LoadLocal, *idx as i32)
                    );
                } else {
                    self.module.functions.last_mut().unwrap().emit(
                        Instruction::with_operand(Opcode::LoadGlobal, 0)
                    );
                }
            }
            crate::ast::Expr::Binary { left, op, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                let opcode = self.bin_op_code(op);
                self.module.functions.last_mut().unwrap().emit(Instruction::new(opcode));
            }
            crate::ast::Expr::Unary { op, expr } => {
                self.compile_expr(expr);
                let opcode = self.unary_op_code(op);
                self.module.functions.last_mut().unwrap().emit(Instruction::new(opcode));
            }
            crate::ast::Expr::Call { func, args } => {
                for arg in args {
                    self.compile_expr(&arg.expr);
                }
                self.compile_expr(func);
                self.module.functions.last_mut().unwrap().emit(
                    Instruction::with_operand(Opcode::Call, args.len() as i32)
                );
            }
            crate::ast::Expr::If { condition, then_branch, elif_branches, else_branch } => {
                self.compile_expr(condition);
                self.module.functions.last_mut().unwrap().emit(Instruction::new(Opcode::JmpIfNot));
                
                self.compile_expr(then_branch);
                self.module.functions.last_mut().unwrap().emit(Instruction::new(Opcode::Pop));
                
                if let Some(else_b) = else_branch {
                    self.compile_expr(else_b);
                }
            }
            crate::ast::Expr::Lambda { params, body } => {
                self.compile_function("lambda", params, Some(body));
                self.module.functions.last_mut().unwrap().emit(Instruction::new(Opcode::Closure));
            }
            crate::ast::Expr::Array(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr);
                }
                self.module.functions.last_mut().unwrap().emit(
                    Instruction::with_operand(Opcode::CreateArray, exprs.len() as i32)
                );
            }
            crate::ast::Expr::Tuple(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr);
                }
                self.module.functions.last_mut().unwrap().emit(
                    Instruction::with_operand(Opcode::CreateTuple, exprs.len() as i32)
                );
            }
            _ => {}
        }
    }

    fn compile_function(&mut self, name: &str, params: &[crate::ast::Param], body: Option<&crate::ast::Expr>) {
        let mut func = BytecodeFunction::new(name);
        func.params = params.len();
        
        let old_locals = std::mem::replace(&mut self.locals, HashMap::new());
        for (i, param) in params.iter().enumerate() {
            self.locals.insert(param.name.clone(), i);
        }
        
        if let Some(expr) = body {
            self.compile_expr(expr);
        }
        
        self.module.functions.last_mut().map(|f| f.emit(Instruction::new(Opcode::Return)));
        self.module.functions.push(func);
        
        self.locals = old_locals;
    }

    fn pat_name(&self, pat: &crate::ast::Pat) -> String {
        match pat {
            crate::ast::Pat::Ident(n) => n.clone(),
            _ => "_".to_string(),
        }
    }

    fn literal_value(&self, lit: &crate::ast::Literal) -> Value {
        match lit {
            crate::ast::Literal::Int(n) => Value::Int(*n),
            crate::ast::Literal::Float(f) => Value::Float(*f),
            crate::ast::Literal::Bool(b) => Value::Bool(*b),
            crate::ast::Literal::String(s) => Value::String(s.clone()),
            crate::ast::Literal::Char(c) => Value::Char(*c),
            crate::ast::Literal::Null => Value::None,
            crate::ast::Literal::MultilineString(s) => Value::String(s.clone()),
        }
    }

    fn bin_op_code(&self, op: &crate::ast::BinOp) -> Opcode {
        match op {
            crate::ast::BinOp::Add => Opcode::Add,
            crate::ast::BinOp::Sub => Opcode::Sub,
            crate::ast::BinOp::Mul => Opcode::Mul,
            crate::ast::BinOp::Div => Opcode::Div,
            crate::ast::BinOp::Mod => Opcode::Mod,
            crate::ast::BinOp::Pow => Opcode::Pow,
            crate::ast::BinOp::Eq => Opcode::Eq,
            crate::ast::BinOp::Neq => Opcode::Neq,
            crate::ast::BinOp::Lt => Opcode::Lt,
            crate::ast::BinOp::LtEq => Opcode::LtEq,
            crate::ast::BinOp::Gt => Opcode::Gt,
            crate::ast::BinOp::GtEq => Opcode::GtEq,
            crate::ast::BinOp::And => Opcode::And,
            crate::ast::BinOp::Or => Opcode::Or,
            _ => Opcode::Nop,
        }
    }

    fn unary_op_code(&self, op: &crate::ast::UnOp) -> Opcode {
        match op {
            crate::ast::UnOp::Neg => Opcode::Neg,
            crate::ast::UnOp::Not => Opcode::Not,
            crate::ast::UnOp::BitNot => Opcode::BitNot,
            _ => Opcode::Nop,
        }
    }
}

pub fn compile(program: &Program) -> BytecodeModule {
    BytecodeCompiler::new("main").compile(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler() {
        let program = crate::parser::parse("let x = 42").unwrap();
        let module = compile(&program);
        assert!(!module.functions.is_empty());
    }
}
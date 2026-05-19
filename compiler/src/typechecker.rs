use crate::ast::*;
use crate::lexer::Span;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int, Float, Bool, Str, Char, Byte, Null, Void, Never,
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Generic(String, Vec<Type>),
    Function(Box<Type>, Vec<Type>),
    Union(Vec<Type>),
    Nullable(Box<Type>),
    Custom(String),
    Unit,
    Var(TypeVar),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

impl TypeVar {
    static mut COUNTER: usize = 0;
    pub fn new() -> TypeVar {
        unsafe {
            let v = TypeVar(COUNTER);
            COUNTER += 1;
            v
        }
    }
}

impl Type {
    pub fn make_var() -> Type {
        Type::Var(TypeVar::new())
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Bool | Type::Str | Type::Char | Type::Byte | Type::Null | Type::Void | Type::Never | Type::Unit)
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, Type::Array(_) | Type::Tuple(_) | Type::Map(_, _) | Type::Set(_) | Type::Generic(_, _))
    }
}

pub struct TypeScheme {
    pub vars: Vec<TypeVar>,
    pub ty: Type,
}

pub struct TypeEnv {
    vars: HashMap<String, TypeScheme>,
    stack: Vec<HashMap<String, Type>>,
    impls: HashMap<String, Vec<ImplInfo>>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    interfaces: HashMap<String, InterfaceInfo>,
    traits: HashMap<String, TraitInfo>,
    trait_impls: Vec<TraitImplInfo>,
}

#[derive(Debug, Clone)]
struct StructInfo {
    fields: HashMap<String, Type>,
    methods: HashMap<String, (Vec<Type>, Type)>,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    variants: HashMap<String, Vec<Type>>,
}

#[derive(Debug, Clone)]
struct InterfaceInfo {
    methods: HashMap<String, (Vec<Type>, Type)>,
}

#[derive(Debug, Clone)]
struct TraitInfo {
    methods: HashMap<String, (Vec<Type>, Type)>,
}

#[derive(Debug, Clone)]
struct ImplInfo {
    ty: Type,
    methods: HashMap<String, (Vec<Type>, Type)>,
}

#[derive(Debug, Clone)]
struct TraitImplInfo {
    impl_type: Type,
    trait_name: String,
    methods: HashMap<String, (Vec<Type>, Type)>,
}

impl TypeEnv {
    pub fn new() -> TypeEnv {
        let mut env = TypeEnv {
            vars: HashMap::new(),
            stack: vec![HashMap::new()],
            impls: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            interfaces: HashMap::new(),
            traits: HashMap::new(),
            trait_impls: Vec::new(),
        };
        env.init_primitives();
        env
    }

    fn init_primitives(&mut self) {
        self.push_scope();
        let prims = vec![
            ("Int", Type::Int),
            ("Float", Type::Float),
            ("Bool", Type::Bool),
            ("Str", Type::Str),
            ("Char", Type::Char),
            ("Byte", Type::Byte),
            ("Null", Type::Null),
            ("Void", Type::Void),
            ("Never", Type::Never),
        ];
        for (name, ty) in prims {
            self.stack.last_mut().unwrap().insert(name.to_string(), ty);
        }
    }

    pub fn push_scope(&mut self) {
        self.stack.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.stack.pop();
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.stack.last_mut() {
            scope.insert(name, ty);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.stack.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn insert_struct(&mut self, name: String, info: StructInfo) {
        self.structs.insert(name, info);
    }

    pub fn lookup_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.get(name)
    }

    pub fn insert_enum(&mut self, name: String, info: EnumInfo) {
        self.enums.insert(name, info);
    }

    pub fn lookup_enum(&self, name: &str) -> Option<&EnumInfo> {
        self.enums.get(name)
    }

    pub fn insert_interface(&mut self, name: String, info: InterfaceInfo) {
        self.interfaces.insert(name, info);
    }

    pub fn insert_trait(&mut self, name: String, info: TraitInfo) {
        self.traits.insert(name, info);
    }

    pub fn insert_impl(&mut self, ty: Type, info: ImplInfo) {
        self.impls.entry(type_key(&ty)).or_insert_with(Vec::new).push(info);
    }

    pub fn insert_trait_impl(&mut self, info: TraitImplInfo) {
        self.trait_impls.push(info);
    }
}

fn type_key(ty: &Type) -> String {
    match ty {
        Type::Custom(s) => s.clone(),
        Type::Generic(s, _) => s.clone(),
        _ => format!("{:?}", ty),
    }
}

pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<TypeError>,
    substitutions: HashMap<TypeVar, Type>,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
}

impl TypeError {
    pub fn new(message: String) -> TypeError {
        TypeError { message, span: None, help: None }
    }

    pub fn with_span(mut self, span: Span) -> TypeError {
        self.span = Some(span);
        self
    }

    pub fn with_help(mut self, help: String) -> TypeError {
        self.help = Some(help);
        self
    }
}

impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            env: TypeEnv::new(),
            errors: Vec::new(),
            substitutions: HashMap::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        for stmt in &program.stmts {
            self.check_stmt(stmt)?;
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<Type, TypeError> {
        match stmt {
            Stmt::Let { pat, ty, init } => {
                let inferred = if let Some(ref init) = init {
                    self.check_expr(init)?
                } else {
                    Type::make_var()
                };
                let ann_type = if let Some(ref t) = ty {
                    self.translate_type(t)
                } else {
                    inferred.clone()
                };
                self.check_pat(pat, ann_type.clone())?;
                self.env.insert(var_name(pat), ann_type.clone());
                Ok(Type::Void)
            }
            Stmt::Var { pat, ty, init } => {
                let inferred = if let Some(ref init) = init {
                    self.check_expr(init)?
                } else {
                    Type::make_var()
                };
                let ann_type = if let Some(ref t) = ty {
                    self.translate_type(t)
                } else {
                    inferred.clone()
                };
                self.check_pat(pat, ann_type.clone())?;
                self.env.insert(var_name(pat), ann_type.clone());
                Ok(Type::Void)
            }
            Stmt::Const { pat, ty, init } => {
                let init_type = self.check_expr(init)?;
                let declared = self.translate_type(ty);
                self.unify(&init_type, &declared)?;
                self.env.insert(var_name(pat), declared);
                Ok(Type::Void)
            }
            Stmt::Function(func) => {
                self.env.push_scope();
                self.env.insert("self".to_string(), Type::make_var());
                for param in &func.params {
                    let param_type = param.ty.as_ref()
                        .map(|t| self.translate_type(t))
                        .unwrap_or_else(Type::make_var);
                    self.env.insert(param.name.clone(), param_type);
                }
                let ret_type = func.return_type.as_ref()
                    .map(|t| self.translate_type(t))
                    .unwrap_or(Type::Void);
                if let Some(body) = &func.body {
                    let body_type = self.check_expr(body)?;
                    self.unify(&body_type, &ret_type)?;
                }
                self.env.pop_scope();
                let func_type = Type::Function(Box::new(ret_type), vec![]);
                self.env.insert(func.name.clone(), func_type);
                Ok(Type::Void)
            }
            Stmt::Struct(s) => {
                let mut fields = HashMap::new();
                for field in &s.fields {
                    fields.insert(field.name.clone(), self.translate_type(&field.ty));
                }
                let mut methods = HashMap::new();
                for method in &s.methods {
                    let param_types: Vec<Type> = method.params.iter()
                        .map(|p| p.ty.as_ref().map(|t| self.translate_type(t)).unwrap_or_else(Type::make_var))
                        .collect();
                    let ret_type = method.return_type.as_ref()
                        .map(|t| self.translate_type(t))
                        .unwrap_or(Type::Void);
                    methods.insert(method.name.clone(), (param_types, ret_type));
                }
                self.env.insert_struct(s.name.clone(), StructInfo { fields, methods });
                Ok(Type::Void)
            }
            Stmt::Enum(e) => {
                let mut variants = HashMap::new();
                for var in &e.variants {
                    let fields: Vec<Type> = var.fields.iter()
                        .map(|f| self.translate_type(&f.ty))
                        .collect();
                    variants.insert(var.name.clone(), fields);
                }
                self.env.insert_enum(e.name.clone(), EnumInfo { variants });
                Ok(Type::Void)
            }
            Stmt::Interface(i) => {
                let mut methods = HashMap::new();
                for method in &i.methods {
                    let param_types: Vec<Type> = method.params.iter()
                        .map(|p| p.ty.as_ref().map(|t| self.translate_type(t)).unwrap_or_else(Type::make_var))
                        .collect();
                    let ret_type = method.return_type.as_ref()
                        .map(|t| self.translate_type(t))
                        .unwrap_or(Type::Void);
                    methods.insert(method.name.clone(), (param_types, ret_type));
                }
                self.env.insert_interface(i.name.clone(), InterfaceInfo { methods });
                Ok(Type::Void)
            }
            Stmt::Trait(t) => {
                let mut methods = HashMap::new();
                for method in &t.methods {
                    let param_types: Vec<Type> = method.params.iter()
                        .map(|p| p.ty.as_ref().map(|t| self.translate_type(t)).unwrap_or_else(Type::make_var))
                        .collect();
                    let ret_type = method.return_type.as_ref()
                        .map(|t| self.translate_type(t))
                        .unwrap_or(Type::Void);
                    methods.insert(method.name.clone(), (param_types, ret_type));
                }
                self.env.insert_trait(t.name.clone(), TraitInfo { methods });
                Ok(Type::Void)
            }
            Stmt::Impl(impl_block) => {
                let ty = self.translate_type(&impl_block.ty);
                let mut methods = HashMap::new();
                for method in &impl_block.methods {
                    let param_types: Vec<Type> = method.params.iter()
                        .map(|p| p.ty.as_ref().map(|t| self.translate_type(t)).unwrap_or_else(Type::make_var))
                        .collect();
                    let ret_type = method.return_type.as_ref()
                        .map(|t| self.translate_type(t))
                        .unwrap_or(Type::Void);
                    methods.insert(method.name.clone(), (param_types, ret_type));
                }
                self.env.insert_impl(ty, ImplInfo { ty, methods });
                Ok(Type::Void)
            }
            Stmt::Import(_) => Ok(Type::Void),
            Stmt::Export(_) => Ok(Type::Void),
            Stmt::Test { .. } => Ok(Type::Void),
            Stmt::AiBlock { .. } => Ok(Type::Void),
            Stmt::Assert { cond, .. } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &Type::Bool)?;
                Ok(Type::Void)
            }
            Stmt::Return(e) => {
                if let Some(e) = e {
                    let ty = self.check_expr(e)?;
                    Ok(Type::Never)
                } else {
                    Ok(Type::Void)
                }
            }
            Stmt::Break(_) => Ok(Type::Never),
            Stmt::Continue => Ok(Type::Never),
            Stmt::Expr(e) => self.check_expr(e),
            Stmt::Assign { target, value } => {
                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(Type::Void)
            }
            Stmt::AssignOp { target, op, value } => {
                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                let result = self.bin_op_type(op, &target_type, &value_type)?;
                self.unify(&target_type, &result)?;
                Ok(Type::Void)
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_type(lit)),
            Expr::Ident(name) => {
                self.env.lookup(name).ok_or_else(|| 
                    TypeError::new(format!("undefined variable: {}", name))
                )
            }
            Expr::Binary { left, op, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                self.bin_op_type(op, &left_type, &right_type)
            }
            Expr::Unary { op, expr } => {
                let expr_type = self.check_expr(expr)?;
                self.unary_op_type(op, &expr_type)
            }
            Expr::Call { func, args } => {
                let func_type = self.check_expr(func)?;
                let arg_types: Vec<Type> = args.iter()
                    .map(|a| self.check_expr(&a.expr))
                    .collect::<Result<Vec<_>, _>>()?;
                match func_type {
                    Type::Function(ret, params) => {
                        for (param, arg) in params.iter().zip(arg_types.iter()) {
                            self.unify(param, arg)?;
                        }
                        Ok(*ret)
                    }
                    _ => Err(TypeError::new(format!("not a function: {:?}", func_type)))
                }
            }
            Expr::MethodChain { base, calls } => {
                let base_type = self.check_expr(base)?;
                let mut current_type = base_type;
                for call in calls {
                    let method_type = self.lookup_method(&current_type, &call.name)?;
                    let arg_types: Vec<Type> = call.args.iter()
                        .map(|a| self.check_expr(&a.expr))
                        .collect::<Result<Vec<_>, _>>()?;
                    for (param, arg) in method_type.0.iter().zip(arg_types.iter()) {
                        self.unify(param, arg)?;
                    }
                    current_type = method_type.1;
                }
                Ok(current_type)
            }
            Expr::Index { array, index } => {
                let arr_type = self.check_expr(array)?;
                let idx_type = self.check_expr(index)?;
                self.unify(&idx_type, &Type::Int)?;
                match arr_type {
                    Type::Array(t) => Ok(*t),
                    Type::Tuple(types) => {
                        if let Expr::Literal(Literal::Int(i)) = index.as_ref() {
                            types.get(*i as usize).cloned().ok_or_else(|| 
                                TypeError::new("index out of bounds".to_string())
                            )
                        } else {
                            Ok(Type::make_var())
                        }
                    }
                    _ => Err(TypeError::new(format!("not indexable: {:?}", arr_type)))
                }
            }
            Expr::Field { object, field } => {
                let obj_type = self.check_expr(object)?;
                match obj_type {
                    Type::Custom(name) => {
                        if let Some(info) = self.env.lookup_struct(&name) {
                            info.fields.get(field).cloned().ok_or_else(|| 
                                TypeError::new(format!("no field '{}' on {}", field, name))
                            )
                        } else {
                            Err(TypeError::new(format!("unknown type: {}", name)))
                        }
                    }
                    _ => Err(TypeError::new(format!("not a struct: {:?}", obj_type)))
                }
            }
            Expr::If { condition, then_branch, elif_branches, else_branch } => {
                let cond_type = self.check_expr(condition)?;
                self.unify(&cond_type, &Type::Bool)?;
                let then_type = self.check_expr(then_branch)?;
                for elif in elif_branches {
                    let elif_cond_type = self.check_expr(&elif.condition)?;
                    self.unify(&elif_cond_type, &Type::Bool)?;
                    self.check_expr(&elif.body)?;
                }
                if let Some(else_b) = else_branch {
                    let else_type = self.check_expr(else_b)?;
                    self.unify(&then_type, &else_type)?;
                    Ok(then_type)
                } else {
                    Ok(Type::Void)
                }
            }
            Expr::Match { expr, arms } => {
                let _expr_type = self.check_expr(expr)?;
                let mut result_type = Type::make_var();
                for arm in arms {
                    self.check_pat(&arm.pattern, Type::make_var())?;
                    let arm_type = self.check_expr(&arm.body)?;
                    self.unify(&result_type, &arm_type)?;
                }
                Ok(result_type)
            }
            Expr::For { var, iter, body } => {
                let iter_type = self.check_expr(iter)?;
                let elem_type = match &iter_type {
                    Type::Array(t) => *t.clone(),
                    Type::Custom(name) => {
                        if let Some(info) = self.env.lookup_struct(name) {
                            info.fields.values().next().cloned().unwrap_or_else(Type::make_var)
                        } else {
                            Type::make_var()
                        }
                    }
                    _ => Type::make_var(),
                };
                self.env.insert(var.clone(), elem_type);
                self.check_expr(body)?;
                Ok(Type::Void)
            }
            Expr::While { condition, body } => {
                let cond_type = self.check_expr(condition)?;
                self.unify(&cond_type, &Type::Bool)?;
                self.check_expr(body)?;
                Ok(Type::Void)
            }
            Expr::Lambda { params, body } => {
                self.env.push_scope();
                let param_types: Vec<Type> = params.iter()
                    .map(|p| p.ty.as_ref().map(|t| self.translate_type(t)).unwrap_or_else(Type::make_var))
                    .collect();
                for (param, ty) in params.iter().zip(param_types.iter()) {
                    self.env.insert(param.name.clone(), ty.clone());
                }
                let body_type = self.check_expr(body)?;
                self.env.pop_scope();
                Ok(Type::Function(Box::new(body_type), param_types))
            }
            Expr::Block(stmts) => {
                self.env.push_scope();
                let mut result = Type::Void;
                for stmt in stmts {
                    result = self.check_stmt(stmt)?;
                }
                self.env.pop_scope();
                Ok(result)
            }
            Expr::Tuple(exprs) => {
                let types: Vec<Type> = exprs.iter()
                    .map(|e| self.check_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Tuple(types))
            }
            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    return Ok(Type::Array(Box::new(Type::make_var())));
                }
                let elem_type = self.check_expr(&exprs[0])?;
                for expr in exprs.iter().skip(1) {
                    let t = self.check_expr(expr)?;
                    self.unify(&elem_type, &t)?;
                }
                Ok(Type::Array(Box::new(elem_type)))
            }
            Expr::StringInterp { parts } => {
                for part in parts {
                    if let InterpolPart::Expr(e) = part {
                        self.check_expr(e)?;
                    }
                }
                Ok(Type::Str)
            }
            Expr::Await(e) => {
                let inner = self.check_expr(e)?;
                match inner {
                    Type::Generic(name, args) if name == "Future" => {
                        Ok(args.into_iter().next().unwrap_or(Type::make_var()))
                    }
                    _ => Err(TypeError::new(format!("not awaitable: {:?}", inner)))
                }
            }
            Expr::Spawn { body, .. } => {
                self.check_expr(body)?;
                Ok(Type::make_var())
            }
            Expr::Channel(ty) => Ok(Type::Generic("Channel".to_string(), vec![ty.clone()])),
            Expr::Send { channel, value } => {
                let _chan_type = self.check_expr(channel)?;
                let _val_type = self.check_expr(value)?;
                Ok(Type::Void)
            }
            Expr::Recv(channel) => {
                let chan_type = self.check_expr(channel)?;
                if let Type::Generic(_, args) = chan_type {
                    Ok(args.into_iter().next().unwrap_or(Type::make_var()))
                } else {
                    Ok(Type::make_var())
                }
            }
            Expr::Try { expr, else_branch } => {
                let expr_type = self.check_expr(expr)?;
                if let Some(else_b) = else_branch {
                    let else_type = self.check_expr(else_b)?;
                    self.unify(&expr_type, &else_type)?;
                    Ok(else_type)
                } else {
                    Ok(expr_type)
                }
            }
            Expr::OptionCoalesce { left, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                if let Type::Nullable(inner) = left_type {
                    self.unify(&inner, &right_type)?;
                    Ok(right_type)
                } else {
                    Err(TypeError::new("left side of ?? must be nullable".to_string()))
                }
            }
            Expr::Spread(e) => {
                let ty = self.check_expr(e)?;
                Ok(ty)
            }
        }
    }

    fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::Bool(_) => Type::Bool,
            Literal::Char(_) => Type::Char,
            Literal::String(_) | Literal::MultilineString(_) => Type::Str,
            Literal::Null => Type::Null,
        }
    }

    fn bin_op_type(&self, op: &BinOp, left: &Type, right: &Type) -> Result<Type, TypeError> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
                if left == &Type::Int && right == &Type::Int {
                    Ok(Type::Int)
                } else if (left == &Type::Float || left == &Type::Int) && (right == &Type::Float || right == &Type::Int) {
                    Ok(Type::Float)
                } else if left == &Type::Str || right == &Type::Str {
                    Ok(Type::Str)
                } else {
                    Err(TypeError::new(format!("invalid operands for arithmetic: {:?}, {:?}", left, right)))
                }
            }
            BinOp::Eq | BinOp::Neq => {
                self.unify(left, right)?;
                Ok(Type::Bool)
            }
            BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                if (left == &Type::Int || left == &Type::Float) && (right == &Type::Int || right == &Type::Float) {
                    Ok(Type::Bool)
                } else if left == &Type::Str && right == &Type::Str {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::new(format!("cannot compare: {:?}, {:?}", left, right)))
                }
            }
            BinOp::And | BinOp::Or => {
                self.unify(left, &Type::Bool)?;
                self.unify(right, &Type::Bool)?;
                Ok(Type::Bool)
            }
            BinOp::In => Ok(Type::Bool),
            BinOp::Is => Ok(Type::Bool),
            BinOp::Range => Ok(Type::Array(Box::new(Type::make_var()))),
            BinOp::DotDot => Ok(Type::Array(Box::new(Type::make_var()))),
            _ => Ok(Type::make_var()),
        }
    }

    fn unary_op_type(&self, op: &UnOp, expr: &Type) -> Result<Type, TypeError> {
        match op {
            UnOp::Neg => {
                if expr == &Type::Int || expr == &Type::Float {
                    Ok(expr.clone())
                } else {
                    Err(TypeError::new(format!("cannot negate: {:?}", expr)))
                }
            }
            UnOp::Not => {
                self.unify(expr, &Type::Bool)?;
                Ok(Type::Bool)
            }
            UnOp::BitNot => {
                self.unify(expr, &Type::Int)?;
                Ok(Type::Int)
            }
            UnOp::Ref => Ok(Type::make_var()),
            UnOp::Deref => Ok(Type::make_var()),
        }
    }

    fn lookup_method(&self, ty: &Type, name: &str) -> Result<(Vec<Type>, Type), TypeError> {
        match ty {
            Type::Custom(type_name) => {
                if let Some(info) = self.env.lookup_struct(type_name) {
                    info.methods.get(name).cloned()
                        .or_else(|| {
                            for impl_info in self.env.impls.get(type_name).iter().flat_map(|v| v) {
                                return impl_info.methods.get(name).cloned();
                            }
                            None
                        })
                        .ok_or_else(|| TypeError::new(format!("no method '{}' on {}", name, type_name)))
                } else {
                    Err(TypeError::new(format!("unknown type: {}", type_name)))
                }
            }
            _ => Err(TypeError::new(format!("no methods on {:?}", ty))),
        }
    }

    fn check_pat(&self, pat: &Pat, ty: Type) -> Result<(), TypeError> {
        match pat {
            Pat::Wildcard => Ok(()),
            Pat::Ident(_) => Ok(()),
            Pat::Literal(lit) => {
                let pat_ty = self.literal_type(lit);
                self.unify(&pat_ty, &ty)?;
                Ok(())
            }
            Pat::Tuple(pats) => {
                if let Type::Tuple(tys) = ty {
                    for (p, t) in pats.iter().zip(tys.iter()) {
                        self.check_pat(p, t.clone())?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::new("expected tuple".to_string()))
                }
            }
            Pat::Array(pats, rest) => {
                if let Type::Array(elem_ty) = ty {
                    for pat in pats {
                        self.check_pat(pat, *elem_ty.clone())?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::new("expected array".to_string()))
                }
            }
            Pat::Object(_) => Ok(()),
            Pat::Or(pats) => {
                for p in pats {
                    self.check_pat(p, ty.clone())?;
                }
                Ok(())
            }
            Pat::Range(_, _) => Ok(()),
            Pat::Rest => Ok(()),
            Pat::Type(_) => Ok(()),
        }
    }

    fn translate_type(&self, ty: &ast::Type) -> Type {
        match ty {
            ast::Type::Int => Type::Int,
            ast::Type::Float => Type::Float,
            ast::Type::Bool => Type::Bool,
            ast::Type::Str => Type::Str,
            ast::Type::Char => Type::Char,
            ast::Type::Byte => Type::Byte,
            ast::Type::Null => Type::Null,
            ast::Type::Void => Type::Void,
            ast::Type::Never => Type::Never,
            ast::Type::Array(t) => Type::Array(Box::new(self.translate_type(t))),
            ast::Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.translate_type(t)).collect()),
            ast::Type::Map(k, v) => Type::Map(Box::new(self.translate_type(k)), Box::new(self.translate_type(v))),
            ast::Type::Set(t) => Type::Set(Box::new(self.translate_type(t))),
            ast::Type::Generic(name, args) => Type::Generic(name.clone(), args.iter().map(|t| self.translate_type(t)).collect()),
            ast::Type::Union(types) => Type::Union(types.iter().map(|t| self.translate_type(t)).collect()),
            ast::Type::Nullable(t) => Type::Nullable(Box::new(self.translate_type(t))),
            ast::Type::Custom(name) => Type::Custom(name.clone()),
            ast::Type::Unit => Type::Unit,
        }
    }

    fn unify(&mut self, a: &Type, b: &Type) -> Result<(), TypeError> {
        let a = self.apply_substitutions(a);
        let b = self.apply_substitutions(b);

        if a == b {
            return Ok(());
        }

        match (&a, &b) {
            (Type::Var(v), _) => {
                self.substitutions.insert(*v, b.clone());
                Ok(())
            }
            (_, Type::Var(v)) => {
                self.substitutions.insert(*v, a.clone());
                Ok(())
            }
            (Type::Array(a_inner), Type::Array(b_inner)) => {
                self.unify(a_inner, b_inner)
            }
            (Type::Tuple(a_tys), Type::Tuple(b_tys)) if a_tys.len() == b_tys.len() => {
                for (a_t, b_t) in a_tys.iter().zip(b_tys.iter()) {
                    self.unify(a_t, b_t)?;
                }
                Ok(())
            }
            (Type::Function(a_ret, a_params), Type::Function(b_ret, b_params)) if a_params.len() == b_params.len() => {
                self.unify(a_ret, b_ret)?;
                for (a_p, b_p) in a_params.iter().zip(b_params.iter()) {
                    self.unify(a_p, b_p)?;
                }
                Ok(())
            }
            _ => Err(TypeError::new(format!("type mismatch: {:?} vs {:?}", a, b)))
        }
    }

    fn apply_substitutions(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => self.substitutions.get(v).cloned().map(|t| self.apply_substitutions(&t)).unwrap_or_else(|| ty.clone()),
            Type::Array(t) => Type::Array(Box::new(self.apply_substitutions(t))),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.apply_substitutions(t)).collect()),
            Type::Map(k, v) => Type::Map(Box::new(self.apply_substitutions(k)), Box::new(self.apply_substitutions(v))),
            Type::Set(t) => Type::Set(Box::new(self.apply_substitutions(t))),
            Type::Function(ret, params) => Type::Function(Box::new(self.apply_substitutions(ret)), params.iter().map(|p| self.apply_substitutions(p)).collect()),
            Type::Union(ts) => Type::Union(ts.iter().map(|t| self.apply_substitutions(t)).collect()),
            Type::Nullable(t) => Type::Nullable(Box::new(self.apply_substitutions(t))),
            Type::Generic(name, args) => Type::Generic(name.clone(), args.iter().map(|a| self.apply_substitutions(a)).collect()),
            _ => ty.clone(),
        }
    }
}

fn var_name(pat: &Pat) -> String {
    match pat {
        Pat::Ident(n) => n.clone(),
        _ => "_".to_string(),
    }
}

pub fn type_check(program: &Program) -> Result<(), Vec<TypeError>> {
    TypeChecker::new().check(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_simple_let() {
        let prog = parse("let x = 42").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let prog = parse("let x: Int = 42.0").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_err());
    }

    #[test]
    fn test_function() {
        let prog = parse("fn add(a: Int, b: Int) -> Int: a + b").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_ok());
    }

    #[test]
    fn test_struct() {
        let prog = parse("struct Point:
  x: Float
  y: Float").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_ok());
    }

    #[test]
    fn test_if_expr() {
        let prog = parse("let x = if true: 1 else: 0").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_ok());
    }

    #[test]
    fn test_array() {
        let prog = parse("let arr = [1, 2, 3]").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_ok());
    }

    #[test]
    fn test_lambda() {
        let prog = parse("let add = (a: Int, b: Int) -> Int: a + b").unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check(&prog).is_ok());
    }
}
use crate::lexer::{SpannedToken, Token};

#[derive(Debug, Clone)]
pub enum Type {
    Int, Float, Bool, Str, Char, Byte, Null, Void, Never,
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Generic(String, Vec<Type>),
    Union(Vec<Type>),
    Nullable(Box<Type>),
    Custom(String),
    Unit,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    MultilineString(String),
    Null,
}

#[derive(Debug, Clone)]
pub enum Pat {
    Wildcard,
    Ident(String),
    Literal(Literal),
    Tuple(Vec<Pat>),
    Array(Vec<Pat>, bool),
    Object(Vec<(String, Pat)>),
    Or(Vec<Pat>),
    Range(Option<Box<Literal>>, Option<Box<Literal>>),
    Rest,
    Type(Type),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    Unary { op: UnOp, expr: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Arg> },
    MethodChain { base: Box<Expr>, calls: Vec<MethodCall> },
    Index { array: Box<Expr>, index: Box<Expr> },
    Field { object: Box<Expr>, field: String },
    If { condition: Box<Expr>, then_branch: Box<Expr>, elif_branches: Vec<Elif>, else_branch: Option<Box<Expr>> },
    Match { expr: Box<Expr>, arms: Vec<MatchArm> },
    For { var: String, iter: Box<Expr>, body: Box<Expr> },
    While { condition: Box<Expr>, body: Box<Expr> },
    Lambda { params: Vec<Param>, body: Box<Expr> },
    Block(Vec<Stmt>),
    StringInterp { parts: Vec<InterpolPart> },
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Spread(Box<Expr>),
    Await(Box<Expr>),
    Spawn { body: Box<Expr>, ty: Option<Type> },
    Channel(Type),
    Send { channel: Box<Expr>, value: Box<Expr> },
    Recv(Box<Expr>),
    Try { expr: Box<Expr>, else_branch: Option<Box<Expr>> },
    OptionCoalesce { left: Box<Expr>, right: Box<Expr> },
}

#[derive(Debug, Clone)]
pub enum InterpolPart {
    Literal(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Neq, Lt, LtEq, Gt, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor,
    LShift, RShift,
    In, Is,
    Range, DotDot,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg, Not, BitNot, Ref, Deref,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let { pat: Pat, ty: Option<Type>, init: Option<Expr> },
    Var { pat: Pat, ty: Option<Type>, init: Option<Expr> },
    Const { pat: Pat, ty: Type, init: Expr },
    Return(Option<Expr>),
    Break(Option<Expr>),
    Continue,
    Assign { target: Box<Expr>, value: Box<Expr> },
    AssignOp { target: Box<Expr>, op: BinOp, value: Box<Expr> },
    Function(Function),
    Struct(StructDecl),
    Enum(EnumDecl),
    Interface(InterfaceDecl),
    Trait(TraitDecl),
    Impl(ImplBlock),
    Import(ImportDecl),
    Export(ExportDecl),
    Test { name: String, body: Vec<Stmt> },
    AiBlock { stmts: Vec<Stmt> },
    Assert { cond: Expr, msg: Option<String> },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Box<Expr>>,
    pub is_async: bool,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<String>,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub name: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub struct Elif {
    pub condition: Expr,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub is_pub: bool,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<Variant>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub methods: Vec<Function>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: String,
    pub methods: Vec<Function>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub ty: Type,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: String,
    pub items: Vec<(String, Option<String>)>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct ExportDecl {
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Program { stmts }
    }
}
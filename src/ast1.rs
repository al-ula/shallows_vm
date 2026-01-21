// =============================================================================
// AST1: Parsed / Syntax-only AST
// =============================================================================
// Mirrors grammar closely. No IDs, no symbol resolution.
// Used for: parsing, basic grammar validation, yield placement checks.
// This is the first-stage AST before name resolution and type checking.
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

/// Identifier in snake_case with optional leading underscores.
/// Valid pattern: `[_]*[a-z][a-z0-9_]*`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

/// Top-level program: a sequence of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

// ----------------------------------- Types -----------------------------------

/// Type annotations as written in source.
/// Examples: `uint`, `(float, int)`, `(uint, uint, uint)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    Prim(PrimType),
    Tuple(TupleType),
}

/// Primitive type keyword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimType {
    pub kind: PrimTypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimTypeKind {
    Uint,
    Int,
    Float,
    Bool,
}

/// Tuple type: at least two element types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleType {
    pub elems: Vec<TypeRef>, // len >= 2
    pub span: Span,
}

// ---------------------------------- Patterns ---------------------------------

/// Pattern for variable binding in `let`.
/// Can be a single name or a tuple destructuring pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Ident(Ident),
    Tuple(TuplePattern),
}

/// Tuple pattern: `let x, y = ...;`
/// Must have at least two elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuplePattern {
    pub elems: Vec<Ident>, // len >= 2
    pub span: Span,
}

// --------------------------------- Statements --------------------------------

/// Statement: a top-level or block-level construct that does not produce a value.
/// Grammar: `Stmt := LetStmt | AssignStmt | IfStmt | ReturnStmt | ExprStmt | BlockStmt | ";"`.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    Assign(AssignStmt),
    If(IfStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
    Block(BlockStmt),
    Empty(Span),
}

/// Let statement: declares one or more variables.
/// Grammar: `let Pattern (":" Type)? ("=" Expr)? ";"`
/// Examples:
/// - `let speed: float = 10.0;`
/// - `let friction: float;` (uninitialized)
/// - `let x, y = translate_x(translate);` (destructuring)
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<TypeRef>,
    pub init: Option<Expr>,
    pub span: Span,
}

/// Assignment statement: mutates an existing variable.
/// Grammar: `Ident "=" Expr ";"`
/// Note: globals cannot be assigned (enforced in AST2).
#[derive(Debug, Clone, PartialEq)]
pub struct AssignStmt {
    pub target: Ident,
    pub value: Expr,
    pub span: Span,
}

/// If statement (statement form): used where no value is expected.
/// Grammar: `if "(" Expr ")" BlockStmt ("elif" "(" Expr ")" BlockStmt)* ("else" BlockStmt)? ";"?`
/// Each branch block is a `BlockStmt` (no `yield` allowed inside).
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub if_branch: IfBranch,
    pub elif_branches: Vec<IfBranch>,
    pub else_block: Option<BlockStmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBranch {
    pub cond: Expr,
    pub block: BlockStmt,
    pub span: Span,
}

/// Return statement: exits the function/script.
/// Grammar: `return (Expr ("," Expr)*)? ";"?`
/// Empty `values` means `return;` (void return).
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub values: Vec<Expr>, // empty = return void
    pub span: Span,
}

/// Expression statement: a standalone expression used for side effects.
/// Grammar: `Expr ";"`
/// Examples: `print(x);`, `add();`
#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Block statement: a sequence of statements.
/// Grammar: `{ Stmt* }`
/// Used in `if` branches, function bodies, etc.
/// Cannot contain `yield` (enforced by grammar).
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

// -------------------------------- Expressions --------------------------------

/// Expression: produces a value.
/// Grammar: `Expr := IfExpr | BlockExpr | Compare | Add | Mul | Unary | Call | Primary`
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(Ident),
    Lit(Literal),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    If(IfExpr),
    Block(BlockExpr),
    Paren(ParenExpr),
}

/// Parenthesized expression: `( Expr )`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParenExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

// ---------------------------------- Literals ---------------------------------

/// Literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(IntLit),
    Float(FloatLit),
    Bool(BoolLit),
    String(StringLit),
}

/// Integer literal with optional base and suffix.
/// Grammar: `IntLit := (DecLit | BinLit) [iu]?`
/// - DecLit: `0 | [1-9][0-9_]*`
/// - BinLit: `0b[01][01_]*`
/// Examples: `42`, `0b1010`, `123u`, `0b1111i`
#[derive(Debug, Clone, PartialEq)]
pub struct IntLit {
    pub raw: String, // source text including underscores
    pub base: IntBase,
    pub suffix: Option<IntSuffix>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntBase {
    Dec,
    Bin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSuffix {
    I, // int
    U, // uint
}

/// Float literal with optional suffix.
/// Grammar: `FloatLit := (digits "." digits | digits exponent) [f]?`
/// Examples: `10.0`, `1.5f`, `1e-3`
#[derive(Debug, Clone, PartialEq)]
pub struct FloatLit {
    pub raw: String,
    pub suffix: Option<FloatSuffix>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSuffix {
    F, // float
}

/// Boolean literal, case-insensitive.
/// Grammar: `true | false | True | FALSE | ...`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolLit {
    pub value: bool,
    pub span: Span,
}

/// String literal: `"..."` with escape sequences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLit {
    pub value: String, // unescaped content
    pub span: Span,
}

// ------------------------------- Unary / Binary ------------------------------

/// Unary operation.
/// Grammar: `UnaryExpr := ("-" | "!") Expr`
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -
    Not, // ! (optional)
}

/// Binary operation.
/// Grammar: `BinaryExpr := Expr Op Expr`
/// Precedence (high to low): Call > Unary > Mul/Div > Add/Sub > Compare
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

// ----------------------------------- Call ------------------------------------

/// Function call expression.
/// Grammar: `CallExpr := Ident "(" (Expr ("," Expr)*)? ")"`
/// Examples: `print(x)`, `add()`, `translate_x(translate)`
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

// ------------------------------ If Expression --------------------------------

/// If expression (value-producing form).
/// Grammar: `IfExpr := "if" "(" Expr ")" BlockExpr ("elif" "(" Expr ")" BlockExpr)* ("else" BlockExpr)`
/// Note: `else` is required for expression form (enforced in pass 1).
/// All branches must yield the same type(s) (checked in later passes).
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub if_branch: IfExprBranch,
    pub elif_branches: Vec<IfExprBranch>,
    pub else_block: Option<BlockExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExprBranch {
    pub cond: Box<Expr>, // FIXED: Boxed to prevent infinite recursion (Expr -> IfExpr -> IfExprBranch -> Expr)
    pub block: BlockExpr,
    pub span: Span,
}

// ----------------------------- Block Expression ------------------------------

/// Block expression: yields a value.
/// Grammar: `{ BlockExprItem* }`
/// Must contain at least one `yield` if used in a non-void context.
/// `yield` is only allowed inside `BlockExpr` (not in `BlockStmt`).
#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpr {
    pub items: Vec<BlockExprItem>,
    pub span: Span,
}

/// Item inside a block expression: either a statement or a yield.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockExprItem {
    Stmt(Stmt),
    Yield(YieldStmt),
}

/// Yield statement: produces value(s) from a block expression.
/// Grammar: `yield Expr ("," Expr)* ";"`
/// Only valid inside `BlockExpr`.
#[derive(Debug, Clone, PartialEq)]
pub struct YieldStmt {
    pub values: Vec<Expr>, // len >= 1
    pub span: Span,
}

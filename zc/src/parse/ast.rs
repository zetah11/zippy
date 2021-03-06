use num_bigint::BigUint;

use super::span::{Span, Spanned};

#[derive(Debug)]
pub struct Decl {
    pub node: DeclNode,
    pub doc: Option<String>,
}

#[derive(Debug)]
pub enum DeclNode {
    Constant {
        name: Spanned<String>,
        anno: Option<Spanned<Expr>>,
        body: Spanned<Expr>,
    },
    Function {
        name: Spanned<String>,
        args: Vec<(Spanned<String>, Option<Spanned<Expr>>)>,
        rett: Option<Spanned<Expr>>,
        body: Spanned<Block>,
        type_span: Span,
    },
    Type(Spanned<String>, Spanned<Expr>),

    Invalid,
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Spanned<Stmt>>,
    pub decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Stmt {
    Block(Block),
    Expr(Spanned<Expr>),
    If(Spanned<Expr>, Block, Option<Block>),
    Return(Spanned<Expr>),

    Invalid,
}

#[derive(Debug)]
pub enum Expr {
    Class(Vec<Decl>),
    Call(Box<Spanned<Expr>>, Vec<Spanned<Expr>>),
    Dot(Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    Binary(Spanned<Op>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Unary(Spanned<Op>, Box<Spanned<Expr>>),

    Bool(bool),
    Decimal(String),
    Integer(BigUint),
    Regex(String),
    String(String),
    Name(String),

    Wildcard,

    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    And,
    AndDo,
    Or,
    OrDo,
    Xor,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Upto,
    Thru,

    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    Mod,

    Not,
    Negate,
}

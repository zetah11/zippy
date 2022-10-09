pub mod pretty;
pub use types::Types;

mod types;

use crate::message::Span;
use crate::resolve::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Range(i64, i64),
    Fun(TypeId, TypeId),
    Product(TypeId, TypeId),
    Invalid,
}

#[derive(Debug)]
pub struct Decls {
    pub values: Vec<ValueDef>,
}

#[derive(Debug)]
pub struct ValueDef {
    pub span: Span,
    pub pat: Pat,
    pub bind: Expr,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
    pub typ: TypeId,
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Int(i64),
    Name(Name),
    Lam(Pat, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Tuple(Box<Expr>, Box<Expr>),
    Invalid,
}

#[derive(Clone, Debug)]
pub struct Pat {
    pub node: PatNode,
    pub span: Span,
    pub typ: TypeId,
}

#[derive(Clone, Debug)]
pub enum PatNode {
    Name(Name),
    Tuple(Box<Pat>, Box<Pat>),
    Wildcard,
    Invalid,
}

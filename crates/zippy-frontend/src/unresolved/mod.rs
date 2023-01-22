//! The unresolved HIR is a tree-like representation of the program, where names
//! are still unresolved strings. The tree therefore contains very little
//! semantic information apart from the structure of the code. It is the job of
//! the name resolution pass to take these names and figure out what they refer
//! to.

use zippy_common::hir::BindId;
use zippy_common::message::Span;
use zippy_common::Number;

#[salsa::interned]
pub struct Name {
    #[return_ref]
    pub text: String,
}

#[salsa::tracked]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Decls {
    #[return_ref]
    pub values: Vec<ValueDef>,

    #[return_ref]
    pub types: Vec<TypeDef>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeDef {
    pub span: Span,
    pub id: BindId,
    pub pat: Pat,
    pub anno: Type,
    pub bind: Type,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ValueDef {
    pub span: Span,
    pub id: BindId,
    pub pat: Pat,
    pub implicits: Vec<(Name, Span)>,
    pub anno: Type,
    pub bind: Expr,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExprNode {
    Name(Name),
    Num(Number),

    Lam(BindId, Pat, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Inst(Box<Expr>, Vec<Type>),

    Tuple(Box<Expr>, Box<Expr>),

    Anno(Box<Expr>, Type),

    Hole,
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pat {
    pub node: PatNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PatNode {
    Name(Name),
    Tuple(Box<Pat>, Box<Pat>),
    Anno(Box<Pat>, Type),
    Wildcard,
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    Name(Name),
    Range(Box<Expr>, Box<Expr>),
    Fun(Box<Type>, Box<Type>),
    Product(Box<Type>, Box<Type>),
    Type,
    Wildcard,
    Invalid,
}

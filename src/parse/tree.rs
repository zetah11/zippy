use crate::message::Span;

pub type Name = String;

#[derive(Clone, Debug)]
pub struct Decl {
    pub node: DeclNode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum DeclNode {
    ValueDecl {
        pat: Expr,
        anno: Option<Expr>,
        bind: Option<Expr>,
    },
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Name(Name),
    Int(u64),

    Group(Box<Expr>),

    Range(Span, Box<Expr>, Box<Expr>),
    Fun(Span, Box<Expr>, Box<Expr>),

    Lam(Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),

    Anno(Box<Expr>, Box<Expr>),

    Invalid,
}

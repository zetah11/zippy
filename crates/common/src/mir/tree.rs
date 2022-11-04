use std::collections::HashMap;

use super::TypeId;
use crate::message::Span;
use crate::names::Name;

#[derive(Debug, Default)]
pub struct Decls {
    pub defs: Vec<ValueDef>,

    pub values: HashMap<Name, Value>,
    pub functions: HashMap<Name, (Vec<Name>, ExprSeq)>,
}

impl Decls {
    pub fn new(defs: Vec<ValueDef>) -> Self {
        Self {
            defs,
            values: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ValueDef {
    pub span: Span,
    pub name: Name,
    pub bind: ExprSeq,
}

#[derive(Clone, Debug)]
pub struct ExprSeq {
    pub exprs: Vec<Expr>,
    pub branch: Branch,
    pub span: Span,
    pub ty: TypeId,
}

impl ExprSeq {
    pub fn new(span: Span, ty: TypeId, exprs: Vec<Expr>, branch: Branch) -> Self {
        Self {
            exprs,
            branch,
            span,
            ty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub node: BranchNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum BranchNode {
    Return(Vec<Value>),
    Jump(Name, Value),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Join {
        name: Name,
        param: Name,
        body: ExprSeq,
    },
    Function {
        name: Name,
        params: Vec<Name>,
        body: ExprSeq,
    },
    Apply {
        names: Vec<Name>,
        fun: Name,
        args: Vec<Value>,
    },
    Tuple {
        name: Name,
        values: Vec<Value>,
    },
    Proj {
        name: Name,
        of: Name,
        at: usize,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Value {
    pub node: ValueNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueNode {
    Int(i64),
    Name(Name),
    Invalid,
}
use super::TypeId;
use crate::message::Span;
use crate::resolve::names::Name;

#[derive(Debug)]
pub struct Decls {
    pub values: Vec<ValueDef>,
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
    pub span: Span,
    pub ty: TypeId,
}

impl ExprSeq {
    pub fn new(span: Span, ty: TypeId) -> Self {
        Self {
            exprs: Vec::new(),
            span,
            ty,
        }
    }

    pub(crate) fn push(&mut self, ex: Expr) {
        self.exprs.push(ex);
    }

    pub(crate) fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Expr>,
    {
        self.exprs.extend(it);
    }
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Produce(Value),
    Jump(Name, Value),
    Join {
        name: Name,
        param: Name,
        body: ExprSeq,
    },
    Function {
        name: Name,
        param: Name,
        body: ExprSeq,
    },
    Apply {
        name: Name,
        fun: Name,
        arg: Value,
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

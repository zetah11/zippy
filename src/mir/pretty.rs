use crate::resolve::names::{Actual, Names};

use super::{Expr, ExprNode, ExprSeq, Name, Pat, PatNode, Value};
use pretty::{Arena, DocAllocator, DocBuilder};

pub struct Prettier<'a> {
    names: &'a Names,
    allocator: Arena<'a>,
}

impl<'a> Prettier<'a> {
    pub fn new(names: &'a Names) -> Self {
        Self {
            names,
            allocator: Arena::new(),
        }
    }

    pub fn pretty_exprs(&'a self, expr: &ExprSeq) -> String {
        let doc = self.doc_expr_seq(expr);
        let mut res = Vec::new();
        doc.render(80, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    pub fn pretty_pat(&'a self, pat: &Pat) -> String {
        let doc = self.doc_pat(pat);
        let mut res = Vec::new();
        doc.render(80, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    pub fn pretty_name(&'a self, name: &Name) -> String {
        let doc = self.doc_name(*name);
        let mut res = Vec::new();
        doc.render(80, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    fn doc_expr_seq(&'a self, exprs: &ExprSeq) -> DocBuilder<Arena<'a>> {
        self.allocator
            .intersperse(
                exprs.exprs.iter().map(|expr| self.doc_expr(expr).nest(2)),
                self.allocator.line_().flat_alt("; "),
            )
            .group()
    }

    fn doc_expr(&'a self, expr: &Expr) -> DocBuilder<Arena<'a>> {
        match &expr.node {
            ExprNode::Produce(value) => self.doc_value(value),
            ExprNode::Jump(to, arg) => self
                .allocator
                .text("jump")
                .append(self.allocator.space())
                .append(self.doc_name(*to))
                .append(self.doc_value(arg).parens())
                .group(),
            ExprNode::Join { name, param, body } => self
                .doc_fun("join", name, param)
                .append(self.doc_expr_seq(body))
                .group(),
            ExprNode::Function { name, param, body } => self
                .doc_fun("fun", name, param)
                .append(self.doc_expr_seq(body))
                .parens()
                .group(),
            ExprNode::Apply { name, fun, arg } => self
                .doc_let(name)
                .append(self.doc_name(*fun))
                .append(self.allocator.space())
                .append(self.doc_value(arg))
                .group(),
            ExprNode::Tuple { name, values } => self
                .doc_let(name)
                .append(
                    self.allocator
                        .intersperse(
                            values.iter().map(|val| self.doc_value(val)),
                            self.allocator.text(", "),
                        )
                        .parens(),
                )
                .group(),
            ExprNode::Proj { name, of, at } => self
                .doc_let(name)
                .append(self.doc_name(*of))
                .append(self.allocator.text("."))
                .append(self.allocator.text(format!("{at}")))
                .group(),
        }
    }

    fn doc_fun(&'a self, kw: &'static str, name: &Name, param: &Name) -> DocBuilder<Arena<'a>> {
        self.allocator
            .text(kw)
            .append(self.allocator.space())
            .append(
                self.doc_name(*name)
                    .append(self.allocator.space())
                    .append(self.doc_name(*param)),
            )
            .append(" =")
            .group()
            .append(self.allocator.softline())
    }

    fn doc_let(&'a self, name: &Name) -> DocBuilder<Arena<'a>> {
        self.allocator
            .text("let")
            .append(self.allocator.space())
            .append(self.doc_name(*name))
            .append(" =")
            .group()
            .append(self.allocator.softline())
    }

    fn doc_pat(&'a self, pat: &Pat) -> DocBuilder<Arena<'a>> {
        match &pat.node {
            PatNode::Name(name) => self.doc_name(*name),
            PatNode::Wildcard => self.allocator.text("?"),
            PatNode::Invalid => self.allocator.text("<invalid>"),
        }
    }

    fn doc_value(&'a self, value: &Value) -> DocBuilder<Arena<'a>> {
        match value {
            Value::Int(i) => self.allocator.text(format!("{i}")),
            Value::Name(name) => self.doc_name(*name),
            Value::Invalid => self.allocator.text("<error>"),
        }
    }

    fn doc_name(&'a self, name: Name) -> DocBuilder<Arena<'a>> {
        let path = self.names.get_path(&name);
        match &path.1 {
            Actual::Lit(lit) => self.allocator.text(lit),
            Actual::Generated(id) => self.allocator.text(String::from(*id)),
            Actual::Scope(_) => unreachable!(),
        }
    }
}

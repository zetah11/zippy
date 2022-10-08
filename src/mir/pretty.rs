use crate::resolve::names::{Actual, Names};

use super::{Expr, ExprNode, Name, Pat, PatNode};
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

    pub fn pretty_expr(&'a self, expr: &Expr) -> String {
        let doc = self.doc_expr(expr);
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

    fn doc_expr(&'a self, expr: &Expr) -> DocBuilder<Arena<'a>> {
        match &expr.node {
            ExprNode::Int(v) => self.allocator.text(format!("{v}")),
            ExprNode::Name(name) => self.doc_name(*name),
            ExprNode::Lam(param, body) => self
                .doc_pat(param)
                .append(self.allocator.space())
                .append("=>")
                .append(self.allocator.line())
                .append(self.doc_expr(body))
                .group(),
            ExprNode::App(fun, arg) => self
                .doc_expr(fun)
                .append(self.allocator.space())
                .append(self.doc_expr(arg))
                .parens()
                .group(),
            ExprNode::Invalid => self.allocator.text("<invalid>"),
        }
    }

    fn doc_pat(&'a self, pat: &Pat) -> DocBuilder<Arena<'a>> {
        match pat.node {
            PatNode::Name(name) => self.doc_name(name),
            PatNode::Wildcard => self.allocator.text("?"),
            PatNode::Invalid => self.allocator.text("<invalid>"),
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

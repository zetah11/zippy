use crate::names::{Actual, Names};

use super::tree::{Branch, BranchNode};
use super::{
    Decls, Expr, ExprNode, ExprSeq, Name, Type, TypeId, Types, Value, ValueDef, ValueNode,
};
use pretty::{Arena, DocAllocator, DocBuilder};

pub struct Prettier<'a> {
    names: &'a Names,
    types: &'a Types,
    allocator: Arena<'a>,
    width: usize,
}

impl<'a> Prettier<'a> {
    pub fn new(names: &'a Names, types: &'a Types) -> Self {
        Self {
            names,
            types,
            allocator: Arena::new(),
            width: 80,
        }
    }

    pub fn with_width(self, width: usize) -> Self {
        Self { width, ..self }
    }

    #[must_use]
    pub fn pretty_decls(&'a self, decls: &Decls) -> String {
        let doc = self.doc_decls(decls);
        let mut res = Vec::new();
        doc.render(self.width, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    #[must_use]
    pub fn pretty_exprs(&'a self, expr: &ExprSeq) -> String {
        let doc = self.doc_expr_seq(None, expr);
        let mut res = Vec::new();
        doc.render(self.width, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    #[must_use]
    pub fn pretty_name(&'a self, name: &Name) -> String {
        let doc = self.doc_name(None, name);
        let mut res = Vec::new();
        doc.render(self.width, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    #[must_use]
    pub fn pretty_type(&'a self, ty: &TypeId) -> String {
        let doc = self.doc_type(ty);
        let mut res = Vec::new();
        doc.render(self.width, &mut res).unwrap();
        String::from_utf8(res).unwrap()
    }

    fn doc_decls(&'a self, decls: &Decls) -> DocBuilder<Arena<'a>> {
        self.allocator.intersperse(
            decls
                .defs
                .iter()
                .map(|def| self.doc_valuedef(None, def))
                .chain(decls.values.iter().map(|(name, value)| {
                    self.doc_let(None, name)
                        .append(self.doc_value(Some(name), value))
                        .nest(2)
                }))
                .chain(decls.functions.iter().map(|(name, (param, body))| {
                    self.doc_fun(None, "fun", name, param)
                        .append(self.doc_expr_seq(Some(name), body))
                        .nest(2)
                })),
            self.allocator.hardline(),
        )
    }

    fn doc_valuedef(&'a self, within: Option<&Name>, def: &ValueDef) -> DocBuilder<Arena<'a>> {
        let bind = self.doc_expr_seq(Some(&def.name), &def.bind);
        self.allocator
            .text("let ")
            .append(self.doc_name(within, &def.name))
            .append(self.allocator.text(" ="))
            .append(self.allocator.line().append(bind))
            .nest(2)
    }

    fn doc_type(&'a self, ty: &TypeId) -> DocBuilder<Arena<'a>> {
        match self.types.get(ty) {
            Type::Range(lo, hi) => self.allocator.text(format!("{lo} upto {hi}")),
            Type::Fun(t, u) => self
                .allocator
                .intersperse(
                    t.iter().map(|ty| self.doc_type(ty)),
                    self.allocator.text(", "),
                )
                .parens()
                .append(self.allocator.text(" -> "))
                .append(
                    self.allocator
                        .intersperse(
                            u.iter().map(|ty| self.doc_type(ty)),
                            self.allocator.text(", "),
                        )
                        .parens(),
                ),
            Type::Product(ts) => self.allocator.intersperse(
                ts.iter().map(|t| self.doc_type(t).parens()),
                self.allocator.text(" * "),
            ),
            Type::Invalid => self.allocator.text("<error>"),
        }
    }

    fn doc_expr_seq(&'a self, within: Option<&Name>, exprs: &ExprSeq) -> DocBuilder<Arena<'a>> {
        self.allocator
            .intersperse(
                exprs
                    .exprs
                    .iter()
                    .map(|expr| self.doc_expr(within, expr).nest(2))
                    .chain(std::iter::once(
                        self.doc_branch(within, &exprs.branch).nest(2),
                    )),
                self.allocator.line_().flat_alt("; "),
            )
            .group()
    }

    fn doc_branch(&'a self, within: Option<&Name>, branch: &Branch) -> DocBuilder<Arena<'a>> {
        match &branch.node {
            BranchNode::Return(values) => self
                .allocator
                .text("return")
                .append(self.allocator.space())
                .append(self.allocator.intersperse(
                    values.iter().map(|value| self.doc_value(within, value)),
                    self.allocator.text(", "),
                )),
            BranchNode::Jump(to, arg) => self
                .allocator
                .text("jump")
                .append(self.doc_name(within, to))
                .append(self.doc_value(within, arg).parens()),
        }
    }

    fn doc_expr(&'a self, within: Option<&Name>, expr: &Expr) -> DocBuilder<Arena<'a>> {
        match &expr.node {
            ExprNode::Join { name, param, body } => self
                .doc_fun(within, "join", name, &[*param])
                .append(self.doc_expr_seq(Some(name), body))
                .group(),
            ExprNode::Function { name, params, body } => {
                let fun = self
                    .doc_fun(within, "fun", name, params)
                    .append(self.doc_expr_seq(Some(name), body));

                fun.flat_alt(
                    self.doc_fun(within, "fun", name, params)
                        .append(self.doc_expr_seq(Some(name), body).parens()),
                )
            }
            ExprNode::Apply { names, fun, args } => self
                .allocator
                .text("let")
                .append(self.allocator.space())
                .append(self.allocator.intersperse(
                    names.iter().map(|name| self.doc_name(within, name)),
                    self.allocator.text(", "),
                ))
                .append(self.allocator.text(" = "))
                .group()
                .append(self.allocator.softline())
                .append(self.doc_name(within, fun))
                .append(
                    self.allocator
                        .intersperse(
                            args.iter().map(|arg| self.doc_value(within, arg)),
                            self.allocator.text(", "),
                        )
                        .parens(),
                )
                .group(),
            ExprNode::Tuple { name, values } => self
                .doc_let(within, name)
                .append(
                    self.allocator
                        .intersperse(
                            values.iter().map(|val| self.doc_value(within, val)),
                            self.allocator.text(", "),
                        )
                        .parens(),
                )
                .group(),
            ExprNode::Proj { name, of, at } => self
                .doc_let(within, name)
                .append(self.doc_name(within, of))
                .append(self.allocator.text("."))
                .append(self.allocator.text(format!("{at}")))
                .group(),
        }
    }

    fn doc_fun(
        &'a self,
        within: Option<&Name>,
        kw: &'static str,
        name: &Name,
        params: &[Name],
    ) -> DocBuilder<Arena<'a>> {
        self.allocator
            .text(kw)
            .append(self.allocator.space())
            .append(
                self.doc_name(within, name).append(
                    self.allocator
                        .intersperse(
                            params
                                .iter()
                                .map(|param_name| self.doc_name(Some(name), param_name)),
                            self.allocator.text(", "),
                        )
                        .parens(),
                ),
            )
            .append(" =")
            .group()
            .append(self.allocator.softline())
    }

    fn doc_let(&'a self, within: Option<&Name>, name: &Name) -> DocBuilder<Arena<'a>> {
        self.allocator
            .text("let")
            .append(self.allocator.space())
            .append(self.doc_name(within, name))
            .append(" =")
            .append(self.allocator.space())
    }

    fn doc_value(&'a self, within: Option<&Name>, value: &Value) -> DocBuilder<Arena<'a>> {
        match &value.node {
            ValueNode::Int(i) => self.allocator.text(format!("{i}")),
            ValueNode::Name(name) => self.doc_name(within, name),
            ValueNode::Invalid => self.allocator.text("<error>"),
        }
    }

    fn doc_name(&'a self, within: Option<&Name>, name: &Name) -> DocBuilder<Arena<'a>> {
        if Some(name) == within {
            return self.allocator.nil();
        }

        let path = self.names.get_path(name);
        let preceding = path
            .0
            .as_ref()
            .map(|name| self.doc_name(within, name))
            .unwrap_or_else(|| self.allocator.nil());

        preceding.append(match &path.1 {
            Actual::Lit(lit) => self.allocator.text(format!(".{lit}")),
            Actual::Generated(id) => self.allocator.text(format!(".{}", String::from(*id))),
            Actual::Root => self.allocator.text("root"),
            Actual::Scope(_) => self.allocator.nil(),
        })
    }
}

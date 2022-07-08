use std::collections::HashMap;

use super::scope::{Scope, ScopeId, Scopes};
use super::{DeclData, Declare};
use crate::name::{ActualName, BareData, Name, NameData};
use crate::parse::hir::{Block, Decls, Expr, ExprNode, Stmt, StmtNode, Stmts};
use crate::parse::hir::{TypeDef, ValueDef};
use crate::parse::ParsedData;
use crate::source::SourceId;

pub struct Declarer<'scopes, 'names> {
    scopes: &'scopes mut Scopes,
    names: &'names dyn Declare,
    id: usize,
}

impl<'a, 'b> Declarer<'a, 'b> {
    /// Create a new name declarer.
    pub fn new(scopes: &'a mut Scopes, names: &'b dyn Declare) -> Self {
        Self {
            scopes,
            names,
            id: 0,
        }
    }

    /// Generate a fresh id unique to this `Declarer`.
    fn gen_id(&mut self) -> usize {
        let id = self.id;
        self.id += 1;
        id
    }

    /// Declare the names of a declarative section.
    pub fn declare_decls(
        &mut self,
        decls: Decls<ParsedData>,
        at: (ScopeId, Name),
    ) -> Decls<DeclData> {
        let mut scope = Scope {
            parent: at.0,
            names: Vec::with_capacity(decls.len()),
        };

        let mut types = HashMap::with_capacity(decls.types.len());
        let mut values = HashMap::with_capacity(decls.values.len());

        let scope_id = self.scopes.make_id();

        // Declare types
        for (name, ty) in decls.types {
            let id = NameData::Child(at.1, ActualName::Name(name));
            let id = self.names.intern_name(id);
            let (name, span) = ty.name;
            let name = self.names.intern_bare(BareData(name));

            let doc = ty.doc;

            let body = self.declare_expr(ty.body, (scope_id, id));

            scope.names.push((name, id));
            types.insert(
                name,
                TypeDef {
                    name: (id, span),
                    doc,
                    body,
                },
            );
        }

        // Declare values
        for (name, value) in decls.values {
            let id = NameData::Child(at.1, ActualName::Name(name));
            let id = self.names.intern_name(id);
            let (name, span) = value.name;
            let name = self.names.intern_bare(BareData(name));

            let doc = value.doc;

            let anno = self.declare_expr(value.anno, (scope_id, id));
            let body = self.declare_expr(value.body, (scope_id, id));

            scope.names.push((name, id));
            values.insert(
                name,
                ValueDef {
                    name: (id, span),
                    doc,
                    anno,
                    body,
                },
            );
        }

        self.scopes.add(scope_id, scope);

        Decls {
            scope: scope_id,
            types,
            values,
        }
    }

    /// Declare the names in a block.
    fn declare_block(
        &mut self,
        block: Block<ParsedData>,
        at: (ScopeId, Name),
        source: SourceId,
    ) -> Block<DeclData> {
        let gen_name = self.gen_id();
        let gen_name = NameData::Child(at.1, ActualName::Generated(source, gen_name));
        let gen_name = self.names.intern_name(gen_name);

        let decls = self.declare_decls(block.decls, (at.0, gen_name));

        let gen_name2 = self.gen_id();
        let gen_name = NameData::Child(gen_name, ActualName::Generated(source, gen_name2));
        let gen_name = self.names.intern_name(gen_name);

        let scope_id = self.scopes.make_id();
        let mut scope = Scope {
            parent: at.0,
            names: vec![],
        };

        let stmts = block
            .stmts
            .0
            .into_iter()
            .map(|stmt| self.declare_stmt(stmt, (scope_id, gen_name), &mut scope))
            .collect();

        self.scopes.add(scope_id, scope);
        let stmts = Stmts(stmts, scope_id);

        Block { decls, stmts }
    }

    /// Declare the names in a statement.
    fn declare_stmt(
        &mut self,
        stmt: Stmt<ParsedData>,
        at: (ScopeId, Name),
        _scope: &mut Scope,
    ) -> Stmt<DeclData> {
        let node = match stmt.node {
            StmtNode::Block(b) => StmtNode::Block(self.declare_block(b, at, stmt.span.source)),
            StmtNode::Expr(e) => StmtNode::Expr(self.declare_expr(e, at)),
            StmtNode::If { cond, then, elze } => StmtNode::If {
                cond: self.declare_expr(cond, at),
                then: self.declare_block(then, at, stmt.span.source),
                elze: self.declare_block(elze, at, stmt.span.source),
            },
            StmtNode::Return(e) => StmtNode::Return(self.declare_expr(e, at)),
            StmtNode::Invalid => StmtNode::Invalid,
        };

        Stmt {
            node,
            span: stmt.span,
        }
    }

    /// Declare the names in an expression.
    fn declare_expr(&mut self, expr: Expr<ParsedData>, at: (ScopeId, Name)) -> Expr<DeclData> {
        let node = match expr.node {
            ExprNode::Class(decls) => {
                let gen_name = self.gen_id();
                let gen_name =
                    NameData::Child(at.1, ActualName::Generated(expr.span.source, gen_name));
                let gen_name = self.names.intern_name(gen_name);

                // Declare items
                let decls = self.declare_decls(decls, (at.0, gen_name));
                ExprNode::Class(decls)
            }

            ExprNode::Fun(params, body, ()) => {
                let gen_name = self.gen_id();
                let gen_name =
                    NameData::Child(at.1, ActualName::Generated(expr.span.source, gen_name));
                let gen_name = self.names.intern_name(gen_name);

                let mut scope = Scope {
                    parent: at.0,
                    names: Vec::with_capacity(params.len()),
                };

                let scope_id = self.scopes.make_id();

                // Declare params
                let mut args = Vec::with_capacity(params.len());
                for (name, span) in params {
                    let id = NameData::Child(gen_name, ActualName::Name(name.clone()));
                    let id = self.names.intern_name(id);
                    let name = self.names.intern_bare(BareData(name));

                    scope.names.push((name, id));
                    args.push((id, span));
                }

                // Declare body
                let body = self.declare_block(body, (scope_id, gen_name), expr.span.source);

                self.scopes.add(scope_id, scope);
                ExprNode::Fun(args, body, scope_id)
            }

            ExprNode::Dot(obj, field) => {
                let obj = self.declare_expr(*obj, at);
                let field = self.declare_expr(*field, at);
                ExprNode::Dot(Box::new(obj), Box::new(field))
            }

            ExprNode::Arrow(args, ret) => {
                let args = args
                    .into_iter()
                    .map(|ex| self.declare_expr(ex, at))
                    .collect();
                let ret = Box::new(self.declare_expr(*ret, at));
                ExprNode::Arrow(args, ret)
            }

            ExprNode::Call(func, args) => {
                let func = Box::new(self.declare_expr(*func, at));
                let args = args
                    .into_iter()
                    .map(|ex| self.declare_expr(ex, at))
                    .collect();
                ExprNode::Call(func, args)
            }

            ExprNode::Operator(op) => ExprNode::Operator(op),
            ExprNode::String(v) => ExprNode::String(v),
            ExprNode::Regex(v) => ExprNode::Regex(v),
            ExprNode::Integer(v) => ExprNode::Integer(v),
            ExprNode::Decimal(v) => ExprNode::Decimal(v),
            ExprNode::Bool(v) => ExprNode::Bool(v),
            ExprNode::Name(v) => ExprNode::Name(self.names.intern_bare(BareData(v))),
            ExprNode::Wildcard => ExprNode::Wildcard,
            ExprNode::Invalid => ExprNode::Invalid,
        };

        Expr {
            node,
            span: expr.span,
        }
    }
}

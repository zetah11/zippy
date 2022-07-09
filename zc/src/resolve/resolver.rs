use std::collections::HashMap;

use super::resl::Resolve;
use super::{ResolvedData, ResolvedName};
use crate::declare::DeclData;
use crate::message::{Message, Severity};
use crate::name::{ActualName, Bare, BareData, Name};
use crate::parse::hir::{Block, Decls, Expr, ExprNode, Stmt, StmtNode, Stmts};
use crate::parse::hir::{TypeDef, ValueDef};
use crate::scope::{ScopeId, Scopes};

pub struct Resolver<'scopes, 'names> {
    lexical: &'scopes Scopes<(Bare, Name)>,
    names: &'names dyn Resolve,
    errs: Vec<Message>,
}

impl<'a, 'b> Resolver<'a, 'b> {
    /// Create a new name resolver.
    pub fn new(lexical: &'a Scopes<(Bare, Name)>, names: &'b dyn Resolve) -> Self {
        Self {
            lexical,
            names,
            errs: Vec::new(),
        }
    }

    fn emit(&mut self, err: Message) {
        self.errs.push(err);
    }

    /// Get the errors produced by this resolver.
    pub fn get_errs(&mut self) -> Vec<Message> {
        self.errs.drain(..).collect()
    }

    /// Resolve the names in a declarative section.
    pub fn resolve_decls(
        &mut self,
        decls: Decls<DeclData>,
        potential_field: bool,
    ) -> Decls<ResolvedData> {
        let mut types = HashMap::with_capacity(decls.types.len());
        let mut values = HashMap::with_capacity(decls.values.len());

        for (bare, ty) in decls.types {
            let body = self.resolve_expr(ty.body, potential_field, decls.scope);
            let (name, span) = ty.name;
            let id = ResolvedName::Resolved(name);

            types.insert(
                id,
                TypeDef {
                    name: (name, bare, span),
                    doc: ty.doc,
                    body,
                },
            );
        }

        for (bare, val) in decls.values {
            let anno = self.resolve_expr(val.anno, potential_field, decls.scope);
            let body = self.resolve_expr(val.body, potential_field, decls.scope);
            let (name, span) = val.name;
            let id = ResolvedName::Resolved(name);

            values.insert(
                id,
                ValueDef {
                    name: (name, bare, span),
                    doc: val.doc,
                    anno,
                    body,
                },
            );
        }

        Decls {
            scope: (),
            types,
            values,
        }
    }

    fn resolve_block(
        &mut self,
        block: Block<DeclData>,
        potential_field: bool,
    ) -> Block<ResolvedData> {
        let decls = self.resolve_decls(block.decls, potential_field);
        let stmts = block
            .stmts
            .0
            .into_iter()
            .map(|stmt| self.resolve_stmt(stmt, potential_field, block.stmts.1))
            .collect();
        let stmts = Stmts(stmts, ());

        Block { decls, stmts }
    }

    fn resolve_stmt(
        &mut self,
        stmt: Stmt<DeclData>,
        potential_field: bool,
        at: ScopeId,
    ) -> Stmt<ResolvedData> {
        let node = match stmt.node {
            StmtNode::Block(b) => StmtNode::Block(self.resolve_block(b, potential_field)),
            StmtNode::Expr(e) => StmtNode::Expr(self.resolve_expr(e, potential_field, at)),
            StmtNode::If { cond, then, elze } => StmtNode::If {
                cond: self.resolve_expr(cond, potential_field, at),
                then: self.resolve_block(then, potential_field),
                elze: self.resolve_block(elze, potential_field),
            },
            StmtNode::Return(e) => StmtNode::Return(self.resolve_expr(e, potential_field, at)),
            StmtNode::Invalid => StmtNode::Invalid,
        };

        Stmt {
            node,
            span: stmt.span,
        }
    }

    fn resolve_expr(
        &mut self,
        expr: Expr<DeclData>,
        potential_field: bool,
        at: ScopeId,
    ) -> Expr<ResolvedData> {
        let node = match expr.node {
            ExprNode::Class(decls) => ExprNode::Class(self.resolve_decls(decls, potential_field)),
            ExprNode::Fun(params, body, _id) => {
                let params = params
                    .into_iter()
                    .map(|(name, span)| {
                        let bare = self.names.lookup_intern_name(name).get_actual();
                        let bare = match bare {
                            ActualName::Name(n) => BareData(n),
                            _ => unreachable!(),
                        };
                        let bare = self.names.intern_bare(bare);

                        (name, bare, span)
                    })
                    .collect();

                let body = self.resolve_block(body, potential_field);

                ExprNode::Fun(params, body, ())
            }

            ExprNode::Dot(obj, field) => {
                let obj = self.resolve_expr(*obj, potential_field, at);
                let field = self.resolve_expr(*field, true, at);
                ExprNode::Dot(Box::new(obj), Box::new(field))
            }

            ExprNode::Arrow(from, to) => {
                let from = from
                    .into_iter()
                    .map(|t| self.resolve_expr(t, potential_field, at))
                    .collect();
                let to = self.resolve_expr(*to, potential_field, at);
                ExprNode::Arrow(from, Box::new(to))
            }

            ExprNode::Call(func, args) => {
                let func = Box::new(self.resolve_expr(*func, potential_field, at));
                let args = args
                    .into_iter()
                    .map(|ex| self.resolve_expr(ex, potential_field, at))
                    .collect();
                ExprNode::Call(func, args)
            }

            ExprNode::Name(bare) => {
                let mut scope = self.lexical.get(&at);
                let mut id;

                'outer: loop {
                    for (other, name) in scope.names.iter().rev() {
                        if other == &bare {
                            break 'outer ExprNode::Name(if potential_field {
                                ResolvedName::MaybeField {
                                    name: bare,
                                    ifnot: *name,
                                }
                            } else {
                                ResolvedName::Resolved(*name)
                            });
                        }
                    }

                    if let Some(parent) = scope.parent {
                        id = parent;
                        scope = self.lexical.get(&id);
                    } else {
                        let name = self.names.lookup_intern_bare(bare).0;
                        self.emit(Message {
                            severity: Severity::Error,
                            at: expr.span,
                            code: 20,
                            title: format!("unknown name '{name}'"),
                            message: "this name could not be found from the current scope".into(),
                            labels: vec![],
                            notes: vec![],
                        });
                        break ExprNode::Invalid;
                    }
                }
            }

            ExprNode::Operator(op) => ExprNode::Operator(op),
            ExprNode::String(v) => ExprNode::String(v),
            ExprNode::Regex(v) => ExprNode::Regex(v),
            ExprNode::Integer(v) => ExprNode::Integer(v),
            ExprNode::Decimal(v) => ExprNode::Decimal(v),
            ExprNode::Bool(v) => ExprNode::Bool(v),
            ExprNode::Wildcard => ExprNode::Wildcard,
            ExprNode::Invalid => ExprNode::Invalid,
        };

        Expr {
            node,
            span: expr.span,
        }
    }
}

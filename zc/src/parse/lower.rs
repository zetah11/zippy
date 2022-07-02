use super::ast;
use super::hir::{self, TypeDef, ValueDef};
use super::span::Spanned;
use crate::source::{SourceId, Span};

/// The data assosciated with HIR trees after parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedData;
impl hir::HirData for ParsedData {
    type Name = String;
    type Binding = (String, Span);
    type Scope = ();
}

pub fn lower_decls(decls: Vec<ast::Decl>, at: SourceId) -> hir::Decls<ParsedData> {
    let mut res = hir::Decls::default();

    for decl in decls {
        match decl.node {
            ast::DeclNode::Constant {
                name: (name, span),
                anno,
                body,
            } => {
                let span = at.span(span.start, span.end);
                let binding = (name.clone(), span);
                let anno = anno.map(|anno| lower_expr(anno, at)).unwrap_or(hir::Expr {
                    node: hir::ExprNode::Wildcard,
                    span,
                });
                let body = lower_expr(body, at);

                res.values.insert(
                    name,
                    ValueDef {
                        name: binding,
                        doc: decl.doc.map(hir::Doc),
                        anno,
                        body,
                    },
                );
            }

            ast::DeclNode::Function {
                name: (name, name_span),
                args,
                rett,
                body: (body, body_span),
                type_span,
            } => {
                // A definition like `fun f(x: T): U ... end` is converted to a
                // constant definition `let f: fun(T) -> U = fun(x) ... end`
                let binding = (name.clone(), at.span(name_span.start, name_span.end));

                let mut arg_names = Vec::with_capacity(args.len());
                let mut arg_types = Vec::with_capacity(args.len());

                for (arg, anno) in args {
                    let span = at.span(arg.1.start, arg.1.end);
                    arg_names.push((arg.0, span));
                    arg_types.push(anno.map(|anno| lower_expr(anno, at)).unwrap_or(hir::Expr {
                        node: hir::ExprNode::Wildcard,
                        span,
                    }));
                }

                let type_span = at.span(type_span.start, type_span.end);
                let rett = rett.map(|rett| lower_expr(rett, at)).unwrap_or(hir::Expr {
                    node: hir::ExprNode::Wildcard,
                    span: type_span,
                });

                let anno = hir::Expr {
                    span: type_span,
                    node: hir::ExprNode::Arrow(arg_types, Box::new(rett)),
                };

                let body = lower_block(body, at);
                let body = hir::Expr {
                    span: at.span(body_span.start, body_span.end),
                    node: hir::ExprNode::Fun(arg_names, body, ()),
                };

                res.values.insert(
                    name,
                    ValueDef {
                        name: binding,
                        doc: decl.doc.map(hir::Doc),
                        anno,
                        body,
                    },
                );
            }

            ast::DeclNode::Type((name, span), body) => {
                let binding = (name.clone(), at.span(span.start, span.end));
                let body = lower_expr(body, at);

                res.types.insert(
                    name,
                    TypeDef {
                        name: binding,
                        doc: decl.doc.map(hir::Doc),
                        body,
                    },
                );
            }

            ast::DeclNode::Invalid => (),
        }
    }

    res
}

fn lower_block(bl: ast::Block, at: SourceId) -> hir::Block<ParsedData> {
    hir::Block {
        decls: lower_decls(bl.decls, at),
        stmts: hir::Stmts(
            bl.stmts.into_iter().map(|st| lower_stmt(st, at)).collect(),
            (),
        ),
    }
}

fn lower_stmt(st: Spanned<ast::Stmt>, at: SourceId) -> hir::Stmt<ParsedData> {
    let node = match st.0 {
        ast::Stmt::Block(bl) => hir::StmtNode::Block(lower_block(bl, at)),
        ast::Stmt::Expr(ex) => hir::StmtNode::Expr(lower_expr(ex, at)),
        ast::Stmt::If(cond, then, elze) => hir::StmtNode::If {
            cond: lower_expr(cond, at),
            then: lower_block(then, at),
            elze: elze.map_or(
                hir::Block {
                    decls: hir::Decls::default(),
                    stmts: hir::Stmts(vec![], ()),
                },
                |elze| lower_block(elze, at),
            ),
        },
        ast::Stmt::Return(ex) => hir::StmtNode::Return(lower_expr(ex, at)),
        ast::Stmt::Invalid => hir::StmtNode::Invalid,
    };

    hir::Stmt {
        node,
        span: at.span(st.1.start, st.1.end),
    }
}

/// Lower some AST expression.
fn lower_expr(ex: Spanned<ast::Expr>, at: SourceId) -> hir::Expr<ParsedData> {
    let node = match ex.0 {
        ast::Expr::Class(decls) => {
            let decls = lower_decls(decls, at);
            hir::ExprNode::Class(decls)
        }
        ast::Expr::Call(func, args) => {
            let func = lower_expr(*func, at);
            let args = args.into_iter().map(|ex| lower_expr(ex, at)).collect();
            hir::ExprNode::Call(Box::new(func), args)
        }
        ast::Expr::Binary(op, x, y) => {
            let op = lower_binop(op, at);
            let args = vec![lower_expr(*x, at), lower_expr(*y, at)];
            hir::ExprNode::Call(Box::new(op), args)
        }
        ast::Expr::Unary(op, arg) => {
            let op = lower_binop(op, at);
            let arg = lower_expr(*arg, at);
            hir::ExprNode::Call(Box::new(op), vec![arg])
        }
        ast::Expr::Bool(lit) => hir::ExprNode::Bool(lit),
        ast::Expr::Decimal(lit) => hir::ExprNode::Decimal(lit),
        ast::Expr::Integer(lit) => hir::ExprNode::Integer(lit),
        ast::Expr::Regex(lit) => hir::ExprNode::Regex(lit),
        ast::Expr::String(lit) => hir::ExprNode::String(lit),
        ast::Expr::Name(name) => hir::ExprNode::Name(name),
        ast::Expr::Wildcard => hir::ExprNode::Wildcard,
        ast::Expr::Invalid => hir::ExprNode::Invalid,
    };

    hir::Expr {
        node,
        span: at.span(ex.1.start, ex.1.end),
    }
}

/// Lower the binary operators to their HIR equivalents.
fn lower_binop(op: Spanned<ast::Op>, at: SourceId) -> hir::Expr<ParsedData> {
    let new_op = match op.0 {
        ast::Op::And => hir::Operator::And,
        ast::Op::AndDo => hir::Operator::AndDo,
        ast::Op::Or => hir::Operator::Or,
        ast::Op::OrDo => hir::Operator::OrDo,
        ast::Op::Xor => hir::Operator::Xor,

        ast::Op::Equal => hir::Operator::Equal,
        ast::Op::NotEqual => hir::Operator::NotEqual,
        ast::Op::Less => hir::Operator::Less,
        ast::Op::LessEqual => hir::Operator::LessEqual,
        ast::Op::Greater => hir::Operator::Greater,
        ast::Op::GreaterEqual => hir::Operator::GreaterEqual,

        ast::Op::Upto => hir::Operator::Upto,
        ast::Op::Thru => hir::Operator::Thru,

        ast::Op::Add => hir::Operator::Add,
        ast::Op::Subtract => hir::Operator::Subtract,
        ast::Op::Multiply => hir::Operator::Multiply,
        ast::Op::Divide => hir::Operator::Divide,
        ast::Op::Exponent => hir::Operator::Exponent,
        ast::Op::Mod => hir::Operator::Mod,

        ast::Op::Not => hir::Operator::Not,
        ast::Op::Negate => hir::Operator::Negate,
    };

    hir::Expr {
        node: hir::ExprNode::Operator(new_op),
        span: at.span(op.1.start, op.1.end),
    }
}

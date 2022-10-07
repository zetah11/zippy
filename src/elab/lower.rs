use std::collections::HashMap;

use crate::message::Messages;
use crate::mir::{Decls, Expr, ExprNode, Pat, PatNode, Type, TypeId, Types, ValueDef};
use crate::tyck::{self, UniVar};
use crate::Driver;

type HiType = tyck::Type;
type HiPat = tyck::Pat<HiType>;
type HiPatNode = tyck::PatNode;
type HiExpr = tyck::Expr<HiType>;
type HiExprNode = tyck::ExprNode<HiType>;
type HiDecls = tyck::Decls<HiType>;

pub fn lower(
    driver: &mut impl Driver,
    subst: &HashMap<UniVar, HiType>,
    decls: HiDecls,
) -> (Types, Decls) {
    let mut lowerer = Lowerer::new(subst);

    let decls = lowerer.lower_decls(decls);

    driver.report(lowerer.messages);
    (lowerer.types, decls)
}

#[derive(Debug)]
struct Lowerer<'a> {
    types: Types,
    subst: &'a HashMap<UniVar, HiType>,
    messages: Messages,
}

impl<'a> Lowerer<'a> {
    pub fn new(subst: &'a HashMap<UniVar, HiType>) -> Self {
        Self {
            types: Types::new(),
            subst,
            messages: Messages::new(),
        }
    }

    pub fn lower_decls(&mut self, decls: HiDecls) -> Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            let pat = self.lower_pat(def.pat);
            let bind = self.lower_expr(def.bind);

            values.push(ValueDef {
                span: def.span,
                pat,
                bind,
            });
        }

        Decls { values }
    }

    fn lower_expr(&mut self, ex: HiExpr) -> Expr {
        if let HiType::Invalid = ex.data {
            Expr {
                node: ExprNode::Invalid,
                span: ex.span,
                typ: self.types.add(Type::Invalid),
            }
        } else {
            let typ = self.lower_type(ex.data);
            let node = match ex.node {
                HiExprNode::Int(v) => ExprNode::Int(v),

                HiExprNode::Name(name) => ExprNode::Name(name),

                HiExprNode::Lam(param, body) => {
                    let param = self.lower_pat(param);
                    let body = self.lower_expr(*body);
                    ExprNode::Lam(param, Box::new(body))
                }

                HiExprNode::App(fun, arg) => {
                    let fun = self.lower_expr(*fun);
                    let arg = self.lower_expr(*arg);

                    ExprNode::App(Box::new(fun), Box::new(arg))
                }

                HiExprNode::Hole => {
                    self.messages
                        .at(ex.span)
                        .elab_report_hole(format!("{:?}", typ));
                    ExprNode::Invalid
                }
                HiExprNode::Invalid => ExprNode::Invalid,

                HiExprNode::Anno(..) => unreachable!(),
            };

            Expr {
                node,
                span: ex.span,
                typ,
            }
        }
    }

    fn lower_pat(&mut self, pat: HiPat) -> Pat {
        let node = match pat.node {
            HiPatNode::Name(name) => PatNode::Name(name),
            HiPatNode::Wildcard => PatNode::Wildcard,
            HiPatNode::Invalid => PatNode::Invalid,
        };

        Pat {
            node,
            span: pat.span,
            typ: self.lower_type(pat.data),
        }
    }

    fn lower_type(&mut self, ty: HiType) -> TypeId {
        match ty {
            HiType::Fun(t, u) => {
                let t = self.lower_type(*t);
                let u = self.lower_type(*u);

                self.types.add(Type::Fun(t, u))
            }

            HiType::Range(lo, hi) => self.types.add(Type::Range(lo, hi)),

            HiType::Var(v) => {
                if let Some(ty) = self.subst.get(&v).cloned() {
                    self.lower_type(ty)
                } else {
                    unimplemented!()
                }
            }

            HiType::Invalid | HiType::Number => unreachable!(),
        }
    }
}

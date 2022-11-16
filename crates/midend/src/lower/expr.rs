use common::mir::{Block, Branch, BranchNode, Statement, StmtNode, Value, ValueNode};
use common::names::Name;

use super::{HiExpr, HiExprNode, Inst, Lowerer};

impl Lowerer<'_> {
    pub fn lower_expr(&mut self, inst: &Inst, ctx: Name, expr: HiExpr) -> Block {
        let span = expr.span;

        let mut exprs = Vec::new();
        let value = self.make_value(inst, ctx, &mut exprs, expr);
        let ty = value.ty;

        let branch = Branch {
            ty,
            span,
            node: BranchNode::Return(vec![value]),
        };

        Block {
            ty,
            span,
            exprs,
            branch,
        }
    }

    /// Produce a `mir::Value` from an expression. May need to produce several statements `within` a block.
    fn make_value(
        &mut self,
        inst: &Inst,
        ctx: Name,
        within: &mut Vec<Statement>,
        expr: HiExpr,
    ) -> Value {
        let span = expr.span;
        let ty = self.lower_type(inst, expr.data);

        let node = match expr.node {
            HiExprNode::Int(i) => ValueNode::Int(i),
            HiExprNode::Name(name) => ValueNode::Name(name),
            HiExprNode::Invalid => ValueNode::Invalid,
            HiExprNode::Hole => ValueNode::Invalid,

            HiExprNode::Tuple(x, y) => {
                let x = self.make_value(inst, ctx, within, *x);
                let y = self.make_value(inst, ctx, within, *y);

                let name = self.fresh_name(expr.span, ctx, ty);

                let expr = StmtNode::Tuple {
                    name,
                    values: vec![x, y],
                };
                let expr = Statement {
                    ty,
                    span,
                    node: expr,
                };

                within.push(expr);
                ValueNode::Name(name)
            }

            HiExprNode::App(fun, arg) => {
                let fun = self.make_value(inst, ctx, within, *fun);
                let arg = self.make_value(inst, ctx, within, *arg);

                match fun.node {
                    ValueNode::Name(fun) => {
                        let name = self.fresh_name(span, ctx, ty);

                        let expr = StmtNode::Apply {
                            names: vec![name],
                            fun,
                            args: vec![arg],
                        };
                        let expr = Statement {
                            ty,
                            span,
                            node: expr,
                        };

                        within.push(expr);
                        ValueNode::Name(name)
                    }

                    ValueNode::Invalid => ValueNode::Invalid,
                    ValueNode::Int(_) => ValueNode::Invalid,
                }
            }

            HiExprNode::Lam(param, body) => {
                let name = self.fresh_name(expr.span, ctx, ty);

                let mut body = self.lower_expr(inst, name, *body);

                // Insert parameter destructuring
                let (param, mut destructuring) = self.destruct_local(inst, name, param);
                destructuring.extend(body.exprs);
                body.exprs = destructuring;

                let expr = StmtNode::Function {
                    name,
                    params: vec![param],
                    body,
                };
                let expr = Statement {
                    ty,
                    span,
                    node: expr,
                };

                within.push(expr);

                ValueNode::Name(name)
            }

            HiExprNode::Inst(..) => todo!(),

            // Typechecking should remove all annotations
            HiExprNode::Anno(..) => unreachable!(),
        };

        Value { node, span, ty }
    }
}

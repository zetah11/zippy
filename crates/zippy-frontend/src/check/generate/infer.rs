use zippy_common::names::Name;

use super::{bound, constrained, Constrainer};
use crate::check::{Constraint, Type};

impl Constrainer {
    pub(super) fn infer_expr(
        &mut self,
        module: &bound::Module,
        expression: &bound::Expression,
    ) -> constrained::Expression {
        let span = expression.span;
        let (node, data) = match &expression.node {
            bound::ExpressionNode::Entry(entry) => {
                let entry = self.constrain_entry(module, entry);
                let values = entry
                    .names
                    .iter()
                    .map(|name| {
                        let ty = self
                            .context
                            .get(&Name::Item(*name))
                            .expect("all names have been bound")
                            .clone();
                        (*name, ty)
                    })
                    .collect();

                let ty = Type::Trait { values };
                let expr = constrained::ExpressionNode::Entry(entry);
                (expr, ty)
            }

            bound::ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                let pattern = self.constrain_pattern(pattern);
                let body = body
                    .as_ref()
                    .map(|expression| Box::new(self.check_expr(module, expression, anno.clone())));

                let ty = Type::Unit;
                let expr = constrained::ExpressionNode::Let { pattern, body };
                (expr, ty)
            }

            bound::ExpressionNode::Block(exprs, last) => {
                let exprs = exprs
                    .iter()
                    .map(|expression| self.check_expr(module, expression, Type::Unit))
                    .collect();

                let last = Box::new(self.infer_expr(module, last));
                let ty = last.data.clone();
                let expr = constrained::ExpressionNode::Block(exprs, last);
                (expr, ty)
            }

            bound::ExpressionNode::Annotate(expression, ty) => {
                return self.check_expr(module, expression, ty.clone());
            }

            bound::ExpressionNode::Path(expression, field) => {
                let at = field.span;
                let expression = Box::new(self.infer_expr(module, expression));
                let ty = self.fresh(at);

                self.constraints.push(Constraint::Field {
                    at,
                    target: ty.clone(),
                    of: expression.data.clone(),
                    field: field.name,
                });

                let expr = constrained::ExpressionNode::Path(expression, *field);
                (expr, ty)
            }

            bound::ExpressionNode::Name(name) => {
                let ty = self.instantiate(span, name);
                let expr = constrained::ExpressionNode::Name(*name);
                (expr, ty)
            }

            bound::ExpressionNode::Alias(alias) => {
                let ty = self.fresh(span);
                self.constraints
                    .push(Constraint::InstantiatedAlias(span, ty.clone(), *alias));
                let expr = constrained::ExpressionNode::Alias(*alias);
                (expr, ty)
            }

            bound::ExpressionNode::Number(number) => {
                let ty = self.fresh(span);
                self.constraints.push(Constraint::Numeric(span, ty.clone()));
                let expr = constrained::ExpressionNode::Number(*number);
                (expr, ty)
            }

            bound::ExpressionNode::String(string) => {
                let ty = self.fresh(span);
                self.constraints.push(Constraint::Textual(span, ty.clone()));
                let expr = constrained::ExpressionNode::String(*string);
                (expr, ty)
            }

            bound::ExpressionNode::Unit => {
                let ty = self.fresh(span);
                self.constraints
                    .push(Constraint::UnitLike(span, ty.clone()));
                let expr = constrained::ExpressionNode::Unit;
                (expr, ty)
            }

            bound::ExpressionNode::Invalid(reason) => {
                let ty = Type::Invalid(*reason);
                let expr = constrained::ExpressionNode::Invalid(*reason);
                (expr, ty)
            }
        };

        constrained::Expression { span, data, node }
    }
}

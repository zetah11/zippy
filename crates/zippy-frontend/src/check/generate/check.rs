use super::{bound, constrained, Constrainer};
use crate::check::{Constraint, Type};

impl Constrainer {
    pub(super) fn check_expr(
        &mut self,
        module: &bound::Module,
        expression: &bound::Expression,
        against: Type,
    ) -> constrained::Expression {
        let span = expression.span;
        let node = match &expression.node {
            bound::ExpressionNode::Block(exprs, last) => {
                let exprs = exprs
                    .iter()
                    .map(|expression| self.check_expr(module, expression, Type::Unit))
                    .collect();
                let last = Box::new(self.check_expr(module, last, against.clone()));
                constrained::ExpressionNode::Block(exprs, last)
            }

            bound::ExpressionNode::Number(number) => {
                self.constraints
                    .push(Constraint::Numeric(span, against.clone()));
                constrained::ExpressionNode::Number(*number)
            }

            bound::ExpressionNode::String(string) => {
                self.constraints
                    .push(Constraint::Textual(span, against.clone()));
                constrained::ExpressionNode::String(*string)
            }

            bound::ExpressionNode::Unit => {
                self.constraints
                    .push(Constraint::UnitLike(span, against.clone()));
                constrained::ExpressionNode::Unit
            }

            _ => {
                let inferred = self.infer_expr(module, expression);
                let id = self.fresh_coercion(span);
                self.constraints.push(Constraint::Assignable {
                    at: span,
                    id,
                    into: against.clone(),
                    from: inferred.data.clone(),
                });

                constrained::ExpressionNode::Coercion(Box::new(inferred), id)
            }
        };

        constrained::Expression {
            span,
            data: against,
            node,
        }
    }
}

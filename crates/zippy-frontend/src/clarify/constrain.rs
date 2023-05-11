use std::collections::HashMap;

use super::{instanced, Clarifier};
use crate::checked;

impl Clarifier {
    pub(super) fn constrain_item(&mut self, item: checked::Item) -> instanced::Item {
        let names = item.names;

        let node = match item.node {
            checked::ItemNode::Bound { body } => {
                let body = self.constrain_expression(body);
                instanced::ItemNode::Bound { body }
            }

            checked::ItemNode::Let { pattern, body } => {
                let mut ties: HashMap<_, _> = names
                    .iter()
                    .map(|name| {
                        let ty = self
                            .item_types
                            .get(name)
                            .expect("all item names are bound before constraining")
                            .ty
                            .clone();
                        (*name, ty)
                    })
                    .collect();

                let pattern = self.constrain_pattern(
                    |name| {
                        Some(ties.remove(&name).expect(
                            "all names defined by item pattern are part of the `names` field",
                        ))
                    },
                    pattern,
                );

                let body = body.map(|expression| {
                    let body = self.constrain_expression(expression);
                    self.equate(pattern.span, pattern.data.clone(), body.data.clone());
                    body
                });
                instanced::ItemNode::Let { pattern, body }
            }
        };

        instanced::Item { names, node }
    }

    fn constrain_expression(&mut self, expression: checked::Expression) -> instanced::Expression {
        let span = expression.span;
        let data = self.fresh_type(expression.data);
        let node = match expression.node {
            checked::ExpressionNode::Entry(entry) => {
                let instance = self.make_entry(entry);
                self.equate_trait(span, data.clone(), instanced::Instance::Concrete(instance));

                instanced::ExpressionNode::Entry(instance)
            }

            checked::ExpressionNode::Let { pattern, body } => {
                let pattern = self.constrain_pattern(|_| None, pattern);

                let body = if let Some(body) = body {
                    let body = Box::new(self.constrain_expression(*body));
                    self.equate(span, data.clone(), body.data.clone());
                    Some(body)
                } else {
                    None
                };

                instanced::ExpressionNode::Let { pattern, body }
            }

            checked::ExpressionNode::Block(exprs, last) => {
                let exprs = exprs
                    .into_iter()
                    .map(|expression| self.constrain_expression(expression))
                    .collect();
                let last = Box::new(self.constrain_expression(*last));
                self.equate(span, data.clone(), last.data.clone());
                instanced::ExpressionNode::Block(exprs, last)
            }

            checked::ExpressionNode::Path(inner, field) => {
                let inner = Box::new(self.constrain_expression(*inner));
                instanced::ExpressionNode::Path(inner, field)
            }

            checked::ExpressionNode::Coerce(inner) => {
                let inner = Box::new(self.constrain_expression(*inner));
                self.equate(span, data.clone(), inner.data.clone());
                instanced::ExpressionNode::Coerce(inner)
            }

            checked::ExpressionNode::Item(item) => instanced::ExpressionNode::Item(item),

            checked::ExpressionNode::Name(name) => {
                let ty = self.instantiate(name).expect("all names have a type");
                self.equate(span, data.clone(), ty);
                instanced::ExpressionNode::Name(name)
            }

            checked::ExpressionNode::String(string) => instanced::ExpressionNode::String(string),
            checked::ExpressionNode::Number(number) => instanced::ExpressionNode::Number(number),
            checked::ExpressionNode::Invalid(reason) => instanced::ExpressionNode::Invalid(reason),
        };

        instanced::Expression { node, data, span }
    }

    fn constrain_pattern<N, F>(
        &mut self,
        mut name_type: F,
        pattern: checked::Pattern<N>,
    ) -> instanced::Pattern<N>
    where
        N: Copy,
        F: FnMut(N) -> Option<instanced::Type>,
    {
        let span = pattern.span;
        let data = self.fresh_type(pattern.data);
        let (node, data) = match pattern.node {
            checked::PatternNode::Name(name) => {
                let data = name_type(name).unwrap_or(data);
                (instanced::PatternNode::Name(name), data)
            }

            checked::PatternNode::Wildcard => (instanced::PatternNode::Wildcard, data),

            checked::PatternNode::Invalid(reason) => {
                (instanced::PatternNode::Invalid(reason), data)
            }
        };

        instanced::Pattern { node, data, span }
    }

    fn make_entry(&mut self, entry: checked::Entry) -> instanced::InstanceIndex {
        let mut items = Vec::with_capacity(entry.items.len());

        for item in entry.items {
            items.push(item);
        }

        let entry = instanced::EntryInstance { items };
        let id = self.fresh_instance_index();
        assert!(self.instances.insert(id, entry).is_none());

        id
    }
}

//! The application pass is responsible for two things:
//!
//! 1. Applying the type substitution generated by [`solve`](super::solve),
//!    leaving a program without any unification variables inside.
//! 2. "Unwrapping" patterns and leaving only named bindings.

use std::collections::HashMap;

use zippy_common::invalid::Reason;
use zippy_common::names::{ItemName, Name};

use super::types::RangeType;
use super::{constrained, CoercionState, Coercions, Solution, Type, UnifyVar};
use crate::checked;
use crate::flattened::{Module, TypeExpression};
use crate::resolved::Alias;

pub fn apply(program: constrained::Program, solution: Solution) -> checked::Program {
    let mut applier = Applier::new(solution);

    applier.apply_type_exprs(program.type_exprs);
    applier.apply_imports(program.imports);
    applier.apply_items(program.items);

    applier.build()
}

struct Applier {
    substitution: HashMap<UnifyVar, Type>,
    coercions: Coercions,
    delayed: Vec<constrained::DelayedConstraint>,
    aliases: HashMap<Alias, ItemName>,
    type_exprs: HashMap<(Module, TypeExpression), checked::ItemIndex>,
    items: HashMap<constrained::ItemIndex, checked::ItemIndex>,

    program: checked::Program,
}

impl Applier {
    pub fn new(solution: Solution) -> Self {
        Self {
            substitution: solution.substitution,
            coercions: solution.coercions,
            delayed: solution.delayed,
            aliases: solution.aliases,
            type_exprs: HashMap::new(),
            items: HashMap::new(),

            program: checked::Program::default(),
        }
    }

    pub fn build(mut self) -> checked::Program {
        self.apply_constraints();
        self.program
    }

    pub fn apply_imports(&mut self, imports: constrained::Imports) {
        for import in imports.into_iter() {
            let from = self.apply_expression(import.from);
            let _ = import.names;

            let pattern = checked::PatternNode::Wildcard;
            let pattern = checked::Pattern {
                node: pattern,
                data: from.data.clone(),
                span: from.span,
            };

            let index = self.make_item();
            let item = checked::Item::Let {
                pattern,
                body: Some(from),
            };

            self.program.add_item_for(index, item);
        }
    }

    pub fn apply_items(&mut self, items: constrained::Items) {
        for index in items.indicies() {
            let new_index = self.make_item();
            self.items.insert(index, new_index);
        }

        for (index, item) in items.into_iter() {
            let new_index = *self.items.get(&index).expect("set by previous loop");
            self.make_item_for(new_index, item);
        }
    }

    pub fn apply_type_exprs(
        &mut self,
        exprs: HashMap<(Module, TypeExpression), constrained::Expression>,
    ) {
        for (key, _) in exprs.iter() {
            let index = self.make_item();
            assert!(self.type_exprs.insert(*key, index).is_none());
        }

        for (key, expression) in exprs {
            let item = *self.type_exprs.get(&key).expect("set by previous loop");
            self.make_bound_for(item, expression);
        }
    }

    fn apply_type(&self, ty: &Type) -> checked::Type {
        match ty {
            Type::Trait { values } => checked::Type::Trait {
                values: values.iter().map(|(name, _)| *name).collect(),
            },

            Type::Range(range) => checked::Type::Range(self.apply_range_type(*range)),

            Type::Unit => todo!("what is the unit type?"),
            Type::Number => checked::Type::Number,

            Type::Var(var) => match self.substitution.get(var) {
                Some(ty) => self.apply_type(ty),
                None => checked::Type::Invalid(Reason::TypeError),
            },

            Type::Invalid(reason) => checked::Type::Invalid(*reason),
        }
    }

    fn apply_range_type(&self, ty: RangeType) -> checked::RangeType {
        let RangeType {
            clusivity,
            lower,
            upper,
            module,
        } = ty;

        let lower = *self
            .type_exprs
            .get(&(module, lower))
            .expect("all type expressions have been applied");

        let upper = *self
            .type_exprs
            .get(&(module, upper))
            .expect("all type expressions have been applied");

        checked::RangeType {
            clusivity,
            lower,
            upper,
        }
    }

    fn apply_expression(&self, expression: constrained::Expression) -> checked::Expression {
        let span = expression.span;
        let data = self.apply_type(&expression.data);
        let node = match expression.node {
            constrained::ExpressionNode::Entry(entry) => {
                checked::ExpressionNode::Entry(self.apply_entry(entry))
            }

            constrained::ExpressionNode::Let { pattern, body } => {
                let pattern = self.apply_pattern(pattern);
                let body = body.map(|expression| Box::new(self.apply_expression(*expression)));
                checked::ExpressionNode::Let { pattern, body }
            }

            constrained::ExpressionNode::Block(exprs, last) => {
                let exprs = exprs
                    .into_iter()
                    .map(|expression| self.apply_expression(expression))
                    .collect();
                let last = Box::new(self.apply_expression(*last));
                checked::ExpressionNode::Block(exprs, last)
            }

            constrained::ExpressionNode::Path(of, field) => {
                let of = Box::new(self.apply_expression(*of));
                checked::ExpressionNode::Path(of, field)
            }

            constrained::ExpressionNode::Coercion(inner, id) => match self.coercions.get(&id) {
                CoercionState::Invalid => checked::ExpressionNode::Invalid(Reason::TypeError),
                CoercionState::Equal => return self.apply_expression(*inner),
                CoercionState::Coercible => {
                    let inner = Box::new(self.apply_expression(*inner));
                    checked::ExpressionNode::Coerce(inner)
                }
            },

            constrained::ExpressionNode::Name(name) => checked::ExpressionNode::Name(name),
            constrained::ExpressionNode::Alias(alias) => match self.aliases.get(&alias) {
                Some(actual) => checked::ExpressionNode::Name(Name::Item(*actual)),
                None => checked::ExpressionNode::Invalid(Reason::NameError),
            },

            constrained::ExpressionNode::Number(number) => checked::ExpressionNode::Number(number),
            constrained::ExpressionNode::String(string) => checked::ExpressionNode::String(string),
            constrained::ExpressionNode::Unit => match self.make_unit_value(&data) {
                Some(unit) => unit,
                None => checked::ExpressionNode::Invalid(Reason::TypeError),
            },
            constrained::ExpressionNode::Invalid(reason) => {
                checked::ExpressionNode::Invalid(reason)
            }
        };

        checked::Expression { node, data, span }
    }

    fn apply_entry(&self, entry: constrained::Entry) -> checked::Entry {
        checked::Entry {
            items: entry
                .items
                .into_iter()
                .map(|item| {
                    *self
                        .items
                        .get(&item)
                        .expect("all items have been given a new index")
                })
                .collect(),
        }
    }

    fn apply_pattern<N>(&self, pattern: constrained::Pattern<N>) -> checked::Pattern<N> {
        let span = pattern.span;
        let data = self.apply_type(&pattern.data);
        let node = match pattern.node {
            constrained::PatternNode::Name(name) => checked::PatternNode::Name(name),
            constrained::PatternNode::Unit => checked::PatternNode::Wildcard, // no information loss
            constrained::PatternNode::Invalid(reason) => checked::PatternNode::Invalid(reason),
        };

        checked::Pattern { node, data, span }
    }

    fn apply_constraints(&mut self) {
        let constraints: Vec<_> = self.delayed.drain(..).collect();
        for constraint in constraints {
            let constraint = self.apply_constraint(constraint);
            self.program.constrain(constraint);
        }
    }

    fn apply_constraint(&self, constraint: constrained::DelayedConstraint) -> checked::Constraint {
        match constraint {
            constrained::DelayedConstraint::Equal { first, second } => {
                let first = self.apply_range_type(first);
                let second = self.apply_range_type(second);
                checked::Constraint::BoundEqual { first, second }
            }

            constrained::DelayedConstraint::Subset { big, small } => {
                let big = self.apply_range_type(big);
                let small = self.apply_range_type(small);
                checked::Constraint::BoundSubset { big, small }
            }

            constrained::DelayedConstraint::Unit(range) => {
                checked::Constraint::BoundUnit(self.apply_range_type(range))
            }

            constrained::DelayedConstraint::UnitOrEmpty(range) => {
                checked::Constraint::BoundUnitOrEmpty(self.apply_range_type(range))
            }
        }
    }

    /// Create a new item index.
    fn make_item(&mut self) -> checked::ItemIndex {
        self.program.reserve_item()
    }

    /// Apply a type bound for the given index.
    fn make_bound_for(&mut self, index: checked::ItemIndex, expression: constrained::Expression) {
        let body = self.apply_expression(expression);
        self.program
            .add_item_for(index, checked::Item::Bound { body });
    }

    /// Apply an item for the given item index.
    fn make_item_for(&mut self, index: checked::ItemIndex, item: constrained::Item) {
        let item = match item {
            constrained::Item::Let { pattern, body } => {
                let pattern = self.apply_pattern(pattern);
                let body = body.map(|expression| self.apply_expression(expression));
                checked::Item::Let { pattern, body }
            }
        };

        self.program.add_item_for(index, item);
    }

    /// Create an appropriate unit value for the given type, if such a thing is
    /// reasonable.
    fn make_unit_value(&self, ty: &checked::Type) -> Option<checked::ExpressionNode> {
        match ty {
            // The unit value of a trait exists only if the trait itself is empty
            // TODO: or if all the values are of a unit type.
            checked::Type::Trait { values } if values.is_empty() => {
                Some(checked::ExpressionNode::Entry(checked::Entry {
                    items: Vec::new(),
                }))
            }

            checked::Type::Trait { .. } => None,

            // The unit value of a range is always the lower bound
            checked::Type::Range(checked::RangeType { lower, .. }) => {
                Some(checked::ExpressionNode::Item(*lower))
            }

            // The number type is conceptually infinite (meaning there's more
            // than one element to pick from)
            checked::Type::Number => None,

            // An invalid type might as well have an invalid value as its unit
            // type even if it technically accepts anything.
            checked::Type::Invalid(reason) => Some(checked::ExpressionNode::Invalid(*reason)),
        }
    }
}
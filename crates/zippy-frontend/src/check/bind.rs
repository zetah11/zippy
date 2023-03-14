use std::collections::HashMap;

use zippy_common::names::Name;
use zippy_common::source::{self, Span};

use super::types::{Constraint, Type, UnifyVar};
use crate::flattened::{self, Expression, ExpressionNode, Item, Module, Pattern, PatternNode};
use crate::flattening::flatten_module;
use crate::Db;

#[salsa::tracked]
pub struct Bound {
    #[return_ref]
    pub context: HashMap<Name, Type>,

    #[return_ref]
    pub constraints: Vec<Constraint>,
}

#[salsa::tracked(return_ref)]
pub fn get_bound(db: &dyn Db, module: source::Module) -> Bound {
    let module = flatten_module(db, module);
    let mut binder = Binder::new(db);
    binder.bind_module(&module);
    Bound::new(db, binder.types, binder.constraints)
}

struct Binder<'db> {
    db: &'db dyn Db,
    types: HashMap<Name, Type>,
    counts: HashMap<Span, usize>,
    constraints: Vec<Constraint>,
}

impl<'db> Binder<'db> {
    pub fn new(db: &'db dyn Db) -> Self {
        Self {
            db,
            types: HashMap::new(),
            counts: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    /// Bind the module itself to an approximate type, and do the same for all
    /// of its items.
    pub fn bind_module(&mut self, module: &Module) {
        let entry = module.entry(self.db);
        let mut values = HashMap::new();

        // Bind items
        for item in module.items(self.db).iter() {
            self.bind_item(module, item);
        }

        // Bind type expressions
        for (_, expression) in module.type_exprs(self.db).iter() {
            self.bind_expression(module, expression);
        }

        // Create the trait type for the module
        for item in entry.items.iter() {
            for name in module.items(self.db).names(item) {
                let ty = self
                    .types
                    .get(&Name::Item(name))
                    .expect("all top-level items have been bound");

                values.insert(name, ty.clone());
            }
        }

        let ty = Type::Trait { values };
        let name = Name::Item(module.name(self.db));
        assert!(self.types.insert(name, ty).is_none());
    }

    /// Bind a single item and its subexpressions.
    fn bind_item(&mut self, module: &Module, item: &Item) {
        match item {
            Item::Let {
                pattern,
                anno,
                body,
            } => {
                if let Some(anno) = anno {
                    let ty = self.lower(module, anno);
                    self.bind_pattern(module, pattern, ty);
                } else {
                    let ty = self.fresh(pattern.span);
                    self.bind_pattern(module, pattern, ty);
                }

                if let Some(body) = body {
                    self.bind_expression(module, body);
                }
            }
        }
    }

    /// Bind every name defined in this expression to a type.
    fn bind_expression(&mut self, module: &Module, expression: &Expression) {
        match &expression.node {
            ExpressionNode::Entry(_) => {
                // handled by `self.bind_module`
            }

            ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                if let Some(anno) = anno {
                    let ty = self.lower(module, anno);
                    self.bind_pattern(module, pattern, ty);
                } else {
                    let ty = self.fresh(pattern.span);
                    self.bind_pattern(module, pattern, ty);
                }

                if let Some(body) = body {
                    self.bind_expression(module, body);
                }
            }

            ExpressionNode::Block(exprs) => {
                for expression in exprs {
                    self.bind_expression(module, expression)
                }
            }

            ExpressionNode::Annotate(expression, _anno) => {
                // annotation is handled by `self.bind_module`
                self.bind_expression(module, expression);
            }

            ExpressionNode::Path(expression, _field) => {
                self.bind_expression(module, expression);
            }

            ExpressionNode::Name(_)
            | ExpressionNode::Alias(_)
            | ExpressionNode::Number(_)
            | ExpressionNode::String(_)
            | ExpressionNode::Unit
            | ExpressionNode::Invalid(_) => {}
        }
    }

    /// Bind a pattern to a type.
    fn bind_pattern<N>(&mut self, module: &Module, pattern: &Pattern<N>, ty: Type)
    where
        N: Copy + Into<Name>,
    {
        match &pattern.node {
            PatternNode::Annotate(inner, anno) => {
                let anno = self.lower(module, anno);
                self.constraints
                    .push(Constraint::Equal(pattern.span, anno.clone(), ty));
                self.bind_pattern(module, inner, anno);
            }

            PatternNode::Name(name) => {
                self.types.insert((*name).into(), ty);
            }

            PatternNode::Unit | PatternNode::Invalid(_) => {}
        }
    }

    /// Generate a fresh type at the given span.
    fn fresh(&mut self, at: Span) -> Type {
        let counter = self.counts.entry(at).or_insert(0);
        let count = *counter;
        *counter += 1;

        Type::Var(UnifyVar { span: at, count })
    }

    fn lower(&mut self, module: &Module, ty: &flattened::Type) -> Type {
        match &ty.node {
            flattened::TypeNode::Range {
                clusivity,
                lower,
                upper,
            } => Type::Range {
                clusivity: *clusivity,
                lower: *lower,
                upper: *upper,
                module: *module,
            },

            flattened::TypeNode::Invalid(reason) => Type::Invalid(*reason),
        }
    }
}

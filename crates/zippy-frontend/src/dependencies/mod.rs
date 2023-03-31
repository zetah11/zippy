//! This module is responsible for calculating the direct dependencies of items,
//! which can later be turned into a dependecy graph. An item *depends* on
//! another if the former cannot be evaluated, typechecked, etc. without the
//! latter doing so first.

use std::collections::{HashMap, HashSet};

use zippy_common::components;
use zippy_common::names::{DeclarableName, ItemName};

use crate::checked::{
    Entry, Expression, ExpressionNode, Item, ItemIndex, ItemNode, Pattern, PatternNode, Program,
    RangeType, Type,
};
use crate::Db;

pub struct Dependencies {
    pub graph: HashMap<ItemIndex, HashSet<ItemIndex>>,
    pub order: Vec<HashSet<ItemIndex>>,
    pub names: HashMap<ItemIndex, HashSet<ItemName>>,
}

pub fn get_dependencies(db: &dyn Db, program: &Program) -> Dependencies {
    let dependencies = DependencyFinder::new(db);
    dependencies.find(program)
}

struct DependencyFinder<'db> {
    db: &'db dyn Db,
    graph: HashMap<ItemIndex, HashSet<ItemIndex>>,
    names: HashMap<ItemName, ItemIndex>,
}

impl<'db> DependencyFinder<'db> {
    pub fn new(db: &'db dyn Db) -> Self {
        Self {
            db,
            graph: HashMap::new(),
            names: HashMap::new(),
        }
    }

    pub fn find(mut self, program: &Program) -> Dependencies {
        // Map the item names to the index where they were defined
        for (index, item) in program.items.iter() {
            for name in self.item_names(item) {
                assert!(self.names.insert(name, *index).is_none());
            }
        }

        // Map out the dependency graph
        for (index, item) in program.items.iter() {
            self.for_item(*index, item);
        }

        // Construct the result.
        self.construct()
    }

    fn construct(self) -> Dependencies {
        // Construct the topological ordering
        let order = components::find(&self.graph);

        // Construct the index-to-name map
        let mut names: HashMap<_, HashSet<_>> = HashMap::new();
        for (name, index) in self.names {
            names.entry(index).or_default().insert(name);
        }

        Dependencies {
            graph: self.graph,
            order,
            names,
        }
    }

    /// Get the names introduced by this item.
    fn item_names(&mut self, item: &Item) -> HashSet<ItemName> {
        item.names.iter().copied().collect()
    }

    /// Set the dependencies of this item.
    fn for_item(&mut self, index: ItemIndex, item: &Item) {
        let mut depends = HashSet::new();
        match &item.node {
            ItemNode::Bound { body } => {
                self.for_expression(&mut depends, body);
            }

            ItemNode::Let { pattern, body } => {
                self.for_pattern(&mut depends, pattern);
                if let Some(expression) = body {
                    self.for_expression(&mut depends, expression);
                }
            }
        }

        assert!(self.graph.insert(index, depends).is_none());
    }

    fn for_type(&self, within: &mut HashSet<ItemIndex>, ty: &Type) {
        match ty {
            Type::Trait { values } => {
                // TODO: is this right?
                within.extend(values.iter().filter_map(|name| self.names.get(name)))
            }

            Type::Range(RangeType {
                clusivity: _,
                lower,
                upper,
            }) => {
                within.extend([lower, upper]);
            }

            Type::Number | Type::Invalid(_) => {}
        }
    }

    fn for_expression(&self, within: &mut HashSet<ItemIndex>, expression: &Expression) {
        self.for_type(within, &expression.data);
        match &expression.node {
            ExpressionNode::Entry(Entry { items }) => {
                within.extend(items.iter().copied());
            }

            ExpressionNode::Let { pattern, body } => {
                self.for_pattern(within, pattern);

                if let Some(body) = body {
                    self.for_expression(within, body);
                }
            }

            ExpressionNode::Block(exprs, last) => {
                for expression in exprs {
                    self.for_expression(within, expression);
                }

                self.for_expression(within, last);
            }

            ExpressionNode::Path(expression, _field) => {
                self.for_expression(within, expression);
            }

            ExpressionNode::Coerce(inner) => self.for_expression(within, inner),

            ExpressionNode::Item(item) => {
                within.insert(*item);
            }

            ExpressionNode::Name(name) => {
                if let Some(name) = self.item_name((*name).into()) {
                    within.extend(self.names.get(&name));
                }
            }

            ExpressionNode::Number(_) | ExpressionNode::String(_) | ExpressionNode::Invalid(_) => {}
        }
    }

    /// Get the names referenced by this pattern.
    fn for_pattern<N>(&self, within: &mut HashSet<ItemIndex>, pattern: &Pattern<N>) {
        self.for_type(within, &pattern.data);
        match &pattern.node {
            PatternNode::Name(_) | PatternNode::Wildcard | PatternNode::Invalid(_) => {}
        }
    }

    fn item_name(&self, name: DeclarableName) -> Option<ItemName> {
        if let DeclarableName::Item(name) = name {
            return Some(name);
        }

        match name.parent(<dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(
            self.db,
        )) {
            Some(name) => self.item_name(name),
            None => None,
        }
    }
}

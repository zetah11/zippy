//! This module is responsible for calculating the direct dependencies of items,
//! which can later be turned into a dependecy graph. An item *depends* on
//! another if the former cannot be evaluated, typechecked, etc. without the
//! latter doing so first.

use std::collections::{HashMap, HashSet};

use zippy_common::names::{DeclarableName, ItemName};
use zippy_common::source::Module;

use crate::names::resolve::resolve_module;
use crate::resolved::{
    Alias, Expression, ExpressionNode, Import, Item, Pattern, PatternNode, Type, TypeNode,
};
use crate::Db;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ItemOrAlias {
    Item(ItemName),
    Alias(Alias),
}

#[salsa::tracked]
pub struct ModuleDependencies {
    #[id]
    pub module: Module,

    #[return_ref]
    pub dependencies: HashMap<ItemOrAlias, HashSet<ItemOrAlias>>,
}

#[salsa::tracked]
pub fn get_dependencies(db: &dyn Db, module: Module) -> ModuleDependencies {
    let resolved = resolve_module(db, module);
    let mut dependencies = DependencyFinder::new(db);

    let mut module_dependencies = HashSet::new();

    for part in resolved.parts(db) {
        for import in part.imports.iter() {
            let names = dependencies.import_names(import);
            module_dependencies.extend(names);
        }

        for item in part.items.iter() {
            let names = dependencies.item_names(item);
            module_dependencies.extend(names);
        }
    }

    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let module_name = ItemOrAlias::Item(module.name(zdb));

    assert!(dependencies
        .dependencies
        .insert(module_name, module_dependencies)
        .is_none());

    ModuleDependencies::new(db, module, dependencies.dependencies)
}

struct DependencyFinder<'db> {
    db: &'db dyn Db,
    dependencies: HashMap<ItemOrAlias, HashSet<ItemOrAlias>>,
}

impl<'db> DependencyFinder<'db> {
    pub fn new(db: &'db dyn Db) -> Self {
        Self {
            db,
            dependencies: HashMap::new(),
        }
    }

    /// Get the names created by this import.
    pub fn import_names(&mut self, import: &Import) -> Vec<ItemOrAlias> {
        let mut depends = HashSet::new();
        self.for_expression(&mut depends, &import.from);

        let mut aliases = Vec::with_capacity(import.names.len());
        for name in import.names.iter() {
            let alias = ItemOrAlias::Alias(name.alias);
            self.dependencies.insert(alias, depends.clone());
            aliases.push(alias);
        }

        aliases
    }

    /// Get the names created by this item.
    pub fn item_names(&mut self, item: &Item) -> Vec<ItemOrAlias> {
        match item {
            Item::Let {
                pattern,
                anno,
                body,
            } => {
                let (names, mut depends) = self.pattern_names(pattern);

                if let Some(ty) = anno {
                    self.for_type(&mut depends, ty);
                }

                if let Some(expression) = body {
                    self.for_expression(&mut depends, expression);
                }

                let mut result = Vec::with_capacity(names.len());
                for name in names {
                    let name = ItemOrAlias::Item(name);
                    self.dependencies.insert(name, depends.clone());
                    result.push(name);
                }

                result
            }
        }
    }

    fn for_type(&mut self, within: &mut HashSet<ItemOrAlias>, ty: &Type) {
        match &ty.node {
            TypeNode::Range {
                clusivity: _,
                lower,
                upper,
            } => {
                self.for_expression(within, lower);
                self.for_expression(within, upper);
            }

            TypeNode::Invalid(_) => {}
        }
    }

    fn for_expression(&mut self, within: &mut HashSet<ItemOrAlias>, expression: &Expression) {
        match &expression.node {
            ExpressionNode::Entry { items, imports } => {
                for import in imports {
                    within.extend(self.import_names(import));
                }

                for item in items {
                    within.extend(self.item_names(item));
                }
            }

            ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                let (_, depends) = self.pattern_names(pattern);
                within.extend(depends);

                if let Some(anno) = anno {
                    self.for_type(within, anno);
                }

                if let Some(body) = body {
                    self.for_expression(within, body);
                }
            }

            ExpressionNode::Block(exprs) => {
                for expression in exprs {
                    self.for_expression(within, expression);
                }
            }

            ExpressionNode::Annotate(expression, ty) => {
                self.for_expression(within, expression);
                self.for_type(within, ty);
            }

            ExpressionNode::Path(expression, _field) => {
                self.for_expression(within, expression);
            }

            ExpressionNode::Name(name) => {
                if let Some(name) = self.item_name((*name).into()) {
                    within.insert(ItemOrAlias::Item(name));
                }
            }

            ExpressionNode::Alias(alias) => {
                within.insert(ItemOrAlias::Alias(*alias));
            }

            ExpressionNode::Number(_)
            | ExpressionNode::String(_)
            | ExpressionNode::Unit
            | ExpressionNode::Invalid(_) => {}
        }
    }

    fn pattern_names<N>(&mut self, pattern: &Pattern<N>) -> (Vec<N>, HashSet<ItemOrAlias>)
    where
        N: Copy + Into<DeclarableName>,
    {
        match &pattern.node {
            PatternNode::Annotate(pattern, ty) => {
                let (names, mut depends) = self.pattern_names(pattern);
                self.for_type(&mut depends, ty);
                (names, depends)
            }

            PatternNode::Name(name) => (vec![*name], HashSet::new()),
            PatternNode::Invalid(_) | PatternNode::Unit => (Vec::new(), HashSet::new()),
        }
    }

    fn item_name(&self, name: DeclarableName) -> Option<ItemName> {
        match name {
            DeclarableName::Item(name) => Some(name),
            _ => name
                .parent(<dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(
                    self.db,
                ))
                .and_then(|name| self.item_name(name)),
        }
    }
}

//! This module is responsible for calculating the direct dependencies of items,
//! which can later be turned into a dependecy graph. An item *depends* on
//! another if the former cannot be evaluated, typechecked, etc. without the
//! latter doing so first.

use std::collections::{HashMap, HashSet};

use zippy_common::names::Name;
use zippy_common::source::Module;

use crate::names::resolve::resolve_module;
use crate::resolved::{
    Alias, Expression, ExpressionNode, Import, Item, Pattern, PatternNode, Type, TypeNode,
};
use crate::Db;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NameOrAlias {
    Name(Name),
    Alias(Alias),
}

#[salsa::tracked]
pub struct ModuleDependencies {
    #[id]
    pub module: Module,

    #[return_ref]
    pub dependencies: HashMap<NameOrAlias, HashSet<NameOrAlias>>,
}

#[salsa::tracked]
pub fn get_dependencies(db: &dyn Db, module: Module) -> ModuleDependencies {
    let resolved = resolve_module(db, module);
    let mut dependencies = DependencyFinder::new();

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
    let module_name = NameOrAlias::Name(Name::Item(module.name(zdb)));

    assert!(dependencies
        .dependencies
        .insert(module_name, module_dependencies)
        .is_none());

    ModuleDependencies::new(db, module, dependencies.dependencies)
}

struct DependencyFinder {
    dependencies: HashMap<NameOrAlias, HashSet<NameOrAlias>>,
}

impl DependencyFinder {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Get the names created by this import.
    pub fn import_names(&mut self, import: &Import) -> Vec<NameOrAlias> {
        let mut depends = HashSet::new();
        self.for_expression(&mut depends, &import.from);

        let mut aliases = Vec::with_capacity(import.names.len());
        for name in import.names.iter() {
            let alias = NameOrAlias::Alias(name.alias);
            self.dependencies.insert(alias, depends.clone());
            aliases.push(alias);
        }

        aliases
    }

    /// Get the names created by this item.
    pub fn item_names(&mut self, item: &Item) -> Vec<NameOrAlias> {
        match item {
            Item::Let {
                pattern,
                anno,
                body,
            } => {
                let names = self.pattern_names(pattern, Name::Item);

                let mut depends = HashSet::new();

                if let Some(ty) = anno {
                    self.for_type(&mut depends, ty);
                }

                if let Some(expression) = body {
                    self.for_expression(&mut depends, expression);
                }

                let mut result = Vec::with_capacity(names.len());
                for name in names {
                    let name = NameOrAlias::Name(name);
                    self.dependencies.insert(name, depends.clone());
                    result.push(name);
                }

                result
            }
        }
    }

    fn for_type(&mut self, within: &mut HashSet<NameOrAlias>, ty: &Type) {
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

    fn for_expression(&mut self, within: &mut HashSet<NameOrAlias>, expression: &Expression) {
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
                let mut let_depends = HashSet::new();

                if let Some(anno) = anno {
                    self.for_type(&mut let_depends, anno);
                }

                if let Some(body) = body {
                    self.for_expression(&mut let_depends, body);
                }

                let names = self.pattern_names(pattern, Name::Local);
                for name in names {
                    let name = NameOrAlias::Name(name);
                    self.dependencies.insert(name, let_depends.clone());
                    within.insert(name);
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
                within.insert(NameOrAlias::Name(*name));
            }

            ExpressionNode::Alias(alias) => {
                within.insert(NameOrAlias::Alias(*alias));
            }

            ExpressionNode::Number(_)
            | ExpressionNode::String(_)
            | ExpressionNode::Unit
            | ExpressionNode::Invalid(_) => {}
        }
    }

    fn pattern_names<F, N>(&mut self, pattern: &Pattern<N>, mut f: F) -> Vec<Name>
    where
        F: FnMut(N) -> Name,
        N: Copy,
    {
        match &pattern.node {
            PatternNode::Annotate(pattern, ty) => {
                let names = self.pattern_names(pattern, f);

                let mut depends = HashSet::new();
                self.for_type(&mut depends, ty);

                for name in names.iter() {
                    let name = NameOrAlias::Name(*name);
                    self.dependencies.insert(name, depends.clone());
                }

                names
            }

            PatternNode::Name(name) => vec![f(*name)],
            PatternNode::Invalid(_) | PatternNode::Unit => Vec::new(),
        }
    }
}

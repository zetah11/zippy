use std::collections::HashMap;

use zippy_common::names::Name;
use zippy_common::source::{self, Span};

use super::bound;
use super::types::{Constraint, Template, Type, UnifyVar};
use crate::flattened::{
    self, Entry, Expression, ExpressionNode, ImportIndex, Item, ItemIndex, Module, Pattern,
    PatternNode,
};
use crate::flattening::flatten_module;
use crate::resolved::Alias;
use crate::Db;

#[salsa::tracked]
pub struct Bound {
    #[return_ref]
    pub context: HashMap<Name, Template>,

    #[return_ref]
    pub aliases: HashMap<Alias, Type>,

    #[return_ref]
    pub constraints: Vec<Constraint>,

    #[return_ref]
    pub module: bound::Module,
}

#[salsa::tracked(return_ref)]
pub fn get_bound(db: &dyn Db, module: source::Module) -> Bound {
    let module = flatten_module(db, module);
    let mut binder = Binder::new(db, module);

    let mut type_exprs = HashMap::new();
    for (name, type_expr) in module.type_exprs(db).iter() {
        let type_expr = binder.bind_expression(type_expr);
        type_exprs.insert(name, type_expr);
    }

    let entry = binder.bind_entry(module.entry(db));

    let values = entry
        .items
        .iter()
        .flat_map(|item| binder.items.names(item))
        .map(|item| {
            let ty = binder
                .types
                .get(&Name::Item(item))
                .expect("all names have been bound")
                .clone();
            (item, ty)
        })
        .collect();

    let ty = Template::mono(Type::Trait { values });
    let name = Name::Item(module.name(db));
    binder.types.insert(name, ty);

    let module = bound::Module {
        entry,
        module,
        imports: binder.imports,
        items: binder.items,
        type_exprs,
    };

    Bound::new(db, binder.types, binder.aliases, binder.constraints, module)
}

struct Binder<'db> {
    db: &'db dyn Db,
    module: Module,
    items: bound::Items,
    imports: bound::Imports,

    types: HashMap<Name, Template>,
    aliases: HashMap<Alias, Type>,
    constraints: Vec<Constraint>,

    counts: HashMap<Span, usize>,
}

impl<'db> Binder<'db> {
    pub fn new(db: &'db dyn Db, module: Module) -> Self {
        Self {
            db,
            module,
            items: bound::Items::new(),
            imports: bound::Imports::new(),

            types: HashMap::new(),
            aliases: HashMap::new(),
            constraints: Vec::new(),

            counts: HashMap::new(),
        }
    }

    pub fn bind_entry(&mut self, entry: &Entry) -> bound::Entry {
        let items = entry
            .items
            .iter()
            .map(|item| self.bind_item(item))
            .collect();
        let imports = entry
            .imports
            .iter()
            .map(|import| self.bind_import(import))
            .collect();

        bound::Entry { imports, items }
    }

    /// Bind the names declared by an import.
    fn bind_import(&mut self, import: &ImportIndex) -> bound::ImportIndex {
        let import = self.module.import(self.db, import);

        let from = self.bind_expression(&import.from);
        let names = import
            .names
            .iter()
            .map(|name| {
                let ty = self.fresh(name.span);
                assert!(self.aliases.insert(name.alias, ty.clone()).is_none());
                bound::ImportedName {
                    data: ty,
                    span: name.span,
                    alias: name.alias,
                    name: name.name,
                }
            })
            .collect();

        self.imports.add(bound::Import { from, names })
    }

    /// Bind a single item and its subexpressions.
    fn bind_item(&mut self, item: &ItemIndex) -> bound::ItemIndex {
        let items = self.module.items(self.db);
        let names = items.names(item);
        let item = items.get(item);

        let item = match item {
            Item::Let {
                pattern,
                anno,
                body,
            } => {
                let (pattern, anno) = if let Some(anno) = anno {
                    let ty = self.lower(anno);
                    let pattern = self.bind_pattern(pattern, Template::mono(ty.clone()));
                    (pattern, ty)
                } else {
                    let ty = self.fresh(pattern.span);
                    let pattern = self.bind_pattern(pattern, Template::mono(ty.clone()));
                    (pattern, ty)
                };

                let body = body
                    .as_ref()
                    .map(|expression| self.bind_expression(expression));

                bound::Item::Let {
                    pattern,
                    anno,
                    body,
                }
            }
        };

        self.items.add(names, item)
    }

    /// Bind every name defined in this expression to a type.
    pub fn bind_expression(&mut self, expression: &Expression) -> bound::Expression {
        let span = expression.span;
        let node = match &expression.node {
            ExpressionNode::Entry(entry) => {
                let entry = self.bind_entry(entry);
                bound::ExpressionNode::Entry(entry)
            }

            ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                let (pattern, anno) = if let Some(anno) = anno {
                    let ty = self.lower(anno);
                    let pattern = self.bind_pattern(pattern, Template::mono(ty.clone()));
                    (pattern, ty)
                } else {
                    let ty = self.fresh(pattern.span);
                    let pattern = self.bind_pattern(pattern, Template::mono(ty.clone()));
                    (pattern, ty)
                };

                let body = body
                    .as_ref()
                    .map(|expression| Box::new(self.bind_expression(expression)));

                bound::ExpressionNode::Let {
                    pattern,
                    anno,
                    body,
                }
            }

            ExpressionNode::Block(exprs, last) => {
                let exprs = exprs
                    .iter()
                    .map(|expression| self.bind_expression(expression))
                    .collect();
                let last = Box::new(self.bind_expression(last));
                bound::ExpressionNode::Block(exprs, last)
            }

            ExpressionNode::Annotate(expression, anno) => {
                let expression = Box::new(self.bind_expression(expression));
                let anno = self.lower(anno);
                bound::ExpressionNode::Annotate(expression, anno)
            }

            ExpressionNode::Path(expression, field) => {
                let expression = Box::new(self.bind_expression(expression));
                bound::ExpressionNode::Path(expression, *field)
            }

            ExpressionNode::Name(name) => bound::ExpressionNode::Name(*name),
            ExpressionNode::Alias(alias) => bound::ExpressionNode::Alias(*alias),
            ExpressionNode::Number(number) => bound::ExpressionNode::Number(*number),
            ExpressionNode::String(string) => bound::ExpressionNode::String(*string),
            ExpressionNode::Unit => bound::ExpressionNode::Unit,
            ExpressionNode::Invalid(reason) => bound::ExpressionNode::Invalid(*reason),
        };

        bound::Expression { span, node }
    }

    /// Bind a pattern to a type.
    fn bind_pattern<N>(&mut self, pattern: &Pattern<N>, ty: Template) -> bound::Pattern<N>
    where
        N: Copy + Into<Name>,
    {
        let span = pattern.span;
        let node = match &pattern.node {
            PatternNode::Annotate(inner, anno) => {
                let anno = self.lower(anno);

                self.constraints
                    .push(Constraint::Equal(pattern.span, anno.clone(), ty.ty));

                let anno = Template { ty: anno, ..ty };

                return self.bind_pattern(inner, anno);
            }

            PatternNode::Name(name) => {
                assert!(self.types.insert((*name).into(), ty.clone()).is_none());
                bound::PatternNode::Name(*name)
            }

            PatternNode::Unit => {
                self.constraints
                    .push(Constraint::UnitLike(pattern.span, ty.ty.clone()));
                bound::PatternNode::Unit
            }

            PatternNode::Invalid(reason) => {
                // Equate the type with an invalid node such that we don't get
                // an unsolved type error at this pattern
                self.constraints.push(Constraint::Equal(
                    pattern.span,
                    Type::Invalid(*reason),
                    ty.ty.clone(),
                ));
                bound::PatternNode::Invalid(*reason)
            }
        };

        bound::Pattern {
            span,
            data: ty.ty,
            node,
        }
    }

    /// Generate a fresh type at the given span.
    fn fresh(&mut self, at: Span) -> Type {
        let counter = self.counts.entry(at).or_insert(0);
        let count = *counter;
        *counter += 1;

        Type::Var(UnifyVar { span: at, count })
    }

    fn lower(&mut self, ty: &flattened::Type) -> Type {
        match &ty.node {
            flattened::TypeNode::Range {
                clusivity,
                lower,
                upper,
            } => Type::Range {
                clusivity: *clusivity,
                lower: *lower,
                upper: *upper,
                module: self.module,
            },

            flattened::TypeNode::Invalid(reason) => Type::Invalid(*reason),
        }
    }
}

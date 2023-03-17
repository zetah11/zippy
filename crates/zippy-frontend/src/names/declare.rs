use std::collections::HashMap;

use zippy_common::messages::MessageMaker;
use zippy_common::names::{
    DeclarableName, ItemName, LocalName, Name, RawName, UnnamableName, UnnamableNameKind,
};
use zippy_common::source::{Module, Span};

use crate::ast::{
    AstSource, Expression, ExpressionNode, Import, Item, Pattern, PatternNode, Type, TypeNode,
};
use crate::messages::NameMessages;
use crate::parser::get_ast;
use crate::Db;

/// Get every name declared within this module.
#[salsa::tracked(return_ref)]
pub fn declared_names(db: &dyn Db, module: Module) -> HashMap<Name, Span> {
    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let root = module.name(zdb);
    let mut declarer = Declarer::new(db, DeclarableName::Item(root));

    for source in module.sources(zdb) {
        let source = get_ast(db, *source);
        declarer.declare_source(source);
    }

    declarer.names
}

struct Declarer<'db> {
    db: &'db dyn Db,
    parent: (Vec<DeclarableName>, DeclarableName),
    names: HashMap<Name, Span>,
    imports: HashMap<RawName, Span>,
}

impl<'db> Declarer<'db> {
    pub fn new(db: &'db dyn Db, root: DeclarableName) -> Self {
        Self {
            db,
            parent: (Vec::new(), root),
            names: HashMap::new(),
            imports: HashMap::new(),
        }
    }

    pub fn declare_source(&mut self, source: AstSource) {
        for import in source.imports(self.db) {
            self.declare_import(import);
        }

        for item in source.items(self.db) {
            self.declare_item(item);
        }
    }

    fn declare_import(&mut self, import: &Import) {
        if let Some(from) = &import.from {
            self.locals(self.parent.1, |locals| {
                locals.declare_expression(from, None);
            });
        }

        for name in import.names.iter() {
            let span = name.span;
            let alias = name.alias.name;

            if let Some(previous) = self.imports.get(&alias) {
                // Kinda hacky but shhh
                let name = ItemName::new(self.common_db(), None, alias);
                self.at(span)
                    .duplicate_definition(Name::Item(name), *previous);

                continue;
            }

            self.imports.insert(alias, span);
        }
    }

    fn declare_item(&mut self, item: &Item) {
        match item {
            Item::Let {
                pattern,
                anno,
                body,
            } => {
                let name = self.declare_pattern(pattern);

                let root = match name {
                    Some(name) => DeclarableName::Item(name),
                    None => {
                        let name = UnnamableName::new(
                            self.common_db(),
                            UnnamableNameKind::Pattern,
                            Some(self.parent.1),
                            pattern.span,
                        );
                        DeclarableName::Unnamable(name)
                    }
                };

                self.locals(root, |locals| {
                    if let Some(anno) = anno {
                        locals.declare_type(anno);
                    }

                    if let Some(body) = body {
                        locals.declare_expression(body, name.map(Name::Item))
                    }
                });
            }
        }
    }

    fn declare_pattern(&mut self, pattern: &Pattern) -> Option<ItemName> {
        let span = pattern.span;
        match &pattern.node {
            PatternNode::Annotate(pattern, ty) => {
                let name = self.declare_pattern(pattern);
                let root = match name {
                    Some(root) => DeclarableName::Item(root),
                    None => DeclarableName::Unnamable(UnnamableName::new(
                        self.common_db(),
                        UnnamableNameKind::Pattern,
                        Some(self.parent.1),
                        span,
                    )),
                };

                self.locals(root, |locals| locals.declare_type(ty));

                name
            }

            PatternNode::Name(name) => {
                let name = ItemName::new(self.common_db(), Some(self.parent.1), name.name);
                self.try_declare_name(name, span);
                Some(name)
            }

            PatternNode::Unit => None,
            PatternNode::Invalid(_) => None,
        }
    }

    /// Try to declare a name, and produce an error message if it already has
    /// been declared.
    fn try_declare_name(&mut self, item_name: ItemName, span: Span) {
        let name = Name::Item(item_name);

        if let Some(previous) = self.names.get(&name) {
            self.at(span).duplicate_definition(name, *previous);

            return;
        }

        if let Some(previous) = self.imports.get(&item_name.name(self.common_db())) {
            self.at(span).duplicate_definition(name, *previous);
        }

        if self.common_db().get_module(&item_name).is_some() {
            self.at(span).duplicate_module_definition(name);
        }

        self.names.insert(name, span);
    }

    fn locals<F, T>(&mut self, root: DeclarableName, f: F) -> T
    where
        F: FnOnce(&mut LocalDeclarer) -> T,
    {
        let mut locals = LocalDeclarer::new(self.db, root);

        let result = f(&mut locals);

        for (name, here) in locals.names {
            assert!(
                self.names.insert(name, here).is_none(),
                "nested names should be uniquely nested"
            );
        }

        result
    }

    fn at(&self, span: Span) -> MessageMaker<&'db dyn Db> {
        MessageMaker::new(self.db, span)
    }

    /// Get a database usable with functions from [`zippy_common`].
    fn common_db(&self) -> &'db dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

struct LocalDeclarer<'db> {
    db: &'db dyn Db,
    parent: DeclarableName,
    names: HashMap<Name, Span>,
    scope: usize,
}

impl<'db> LocalDeclarer<'db> {
    fn new(db: &'db dyn Db, root: DeclarableName) -> Self {
        Self {
            db,
            parent: root,
            names: HashMap::new(),
            scope: 0,
        }
    }

    fn declare_type(&mut self, ty: &Type) {
        match &ty.node {
            TypeNode::Range {
                clusivity: _,
                lower,
                upper,
            } => {
                self.declare_expression(lower, None);
                self.declare_expression(upper, None);
            }

            TypeNode::Invalid(_) => {}
        }
    }

    fn declare_expression(&mut self, expression: &Expression, bind: Option<Name>) {
        let span = expression.span;
        match &expression.node {
            ExpressionNode::Entry { items, imports } => {
                let root = match bind {
                    Some(bind) => bind.into(),
                    None => DeclarableName::Unnamable(UnnamableName::new(
                        self.common_db(),
                        UnnamableNameKind::Entry,
                        Some(self.parent),
                        span,
                    )),
                };

                self.items(root, |declarer| {
                    for import in imports {
                        declarer.declare_import(import);
                    }

                    for item in items {
                        declarer.declare_item(item);
                    }
                });
            }

            ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                let bind = self.declare_pattern(pattern);

                if let Some(ty) = anno {
                    self.declare_type(ty);
                }

                if let Some(body) = body {
                    self.declare_expression(body, bind.map(Name::Local));
                }
            }

            ExpressionNode::Block(expressions, last) => {
                for expression in expressions {
                    self.declare_expression(expression, None);
                    self.scope += 1;
                }

                self.declare_expression(last, None);
                self.scope += 1;
            }

            ExpressionNode::Annotate(expression, ty) => {
                self.declare_expression(expression, bind);
                self.declare_type(ty);
            }

            ExpressionNode::Path(expression, _) => {
                self.declare_expression(expression, bind);
            }

            ExpressionNode::Name(_)
            | ExpressionNode::Number(_)
            | ExpressionNode::String(_)
            | ExpressionNode::Unit
            | ExpressionNode::Invalid(_) => {}
        }
    }

    fn declare_pattern(&mut self, pattern: &Pattern) -> Option<LocalName> {
        let span = pattern.span;
        match &pattern.node {
            PatternNode::Annotate(pattern, ty) => {
                self.declare_type(ty);
                self.declare_pattern(pattern)
            }

            PatternNode::Name(name) => {
                let name =
                    LocalName::new(self.common_db(), Some(self.parent), name.name, self.scope);
                self.try_declare_name(name, span);
                Some(name)
            }

            PatternNode::Unit => None,
            PatternNode::Invalid(_) => None,
        }
    }

    /// Attempt to declare a name as being defined at the given span, reporting
    /// an error if it already has been.
    fn try_declare_name(&mut self, name: LocalName, span: Span) {
        let name = Name::Local(name);

        if let Some(previous) = self.names.get(&name) {
            self.at(span).duplicate_definition(name, *previous);
        }

        self.names.insert(name, span);
    }

    fn items<F, T>(&mut self, root: DeclarableName, f: F) -> T
    where
        F: FnOnce(&mut Declarer<'db>) -> T,
    {
        let mut declarer = Declarer::new(self.db, root);

        let result = f(&mut declarer);

        for (name, here) in declarer.names {
            assert!(
                self.names.insert(name, here).is_none(),
                "nested names should be uniquely nested"
            );
        }

        result
    }

    fn at(&self, span: Span) -> MessageMaker<&'db dyn Db> {
        MessageMaker::new(self.db, span)
    }

    fn common_db(&self) -> &'db dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

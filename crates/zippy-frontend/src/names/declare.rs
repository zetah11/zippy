use std::collections::HashMap;

use zippy_common::messages::MessageMaker;
use zippy_common::names::{
    DeclarableName, ItemName, Name, RawName, UnnamableName, UnnamableNameKind,
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
    let mut declarer = Declarer::new(db, root);

    for source in module.sources(zdb) {
        let source = get_ast(db, *source);
        declarer.declare_source(source);
    }

    declarer.names
}

struct Declarer<'db> {
    db: &'db dyn Db,
    scope: (Vec<DeclarableName>, DeclarableName),
    names: HashMap<Name, Span>,
    imports: HashMap<RawName, Span>,
}

impl<'db> Declarer<'db> {
    pub fn new(db: &'db dyn Db, root: ItemName) -> Self {
        let scope = DeclarableName::Item(root);

        Self {
            db,
            scope: (Vec::new(), scope),
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
                anno: _,
                body,
            } => {
                let name = self.declare_pattern(pattern, |declarer, name| {
                    let name = ItemName::new(declarer.common_db(), Some(declarer.scope.1), name);
                    Name::Item(name)
                });

                let name = match name {
                    Some(name) => name.into(),
                    None => {
                        let name = UnnamableName::new(
                            self.common_db(),
                            UnnamableNameKind::Pattern,
                            Some(self.scope.1),
                            pattern.span,
                        );
                        DeclarableName::Unnamable(name)
                    }
                };

                if let Some(body) = body {
                    self.within(name, |declarer| {
                        declarer.declare_expression(body);
                    });
                }
            }
        }
    }

    fn declare_type(&mut self, ty: &Type) {
        match &ty.node {
            TypeNode::Range {
                clusivity: _,
                lower,
                upper,
            } => {
                self.declare_expression(lower);
                self.declare_expression(upper);
            }

            TypeNode::Invalid(_) => {}
        }
    }

    fn declare_expression(&mut self, expression: &Expression) {
        match &expression.node {
            ExpressionNode::Entry { items, imports } => {
                self.nested(|this| {
                    for import in imports {
                        this.declare_import(import);
                    }

                    for item in items {
                        this.declare_item(item);
                    }
                });
            }

            ExpressionNode::Block(expressions) => {
                for expression in expressions {
                    self.declare_expression(expression);
                }
            }

            ExpressionNode::Annotate(expression, ty) => {
                self.declare_expression(expression);
                self.declare_type(ty);
            }

            ExpressionNode::Path(expression, _) => {
                self.declare_expression(expression);
            }

            ExpressionNode::Name(_)
            | ExpressionNode::Number(_)
            | ExpressionNode::String(_)
            | ExpressionNode::Unit
            | ExpressionNode::Invalid(_) => {}
        }
    }

    fn declare_pattern<F>(&mut self, pattern: &Pattern, mut f: F) -> Option<Name>
    where
        F: FnMut(&Self, RawName) -> Name,
    {
        let span = pattern.span;
        match &pattern.node {
            PatternNode::Annotate(pattern, ty) => {
                self.declare_type(ty);
                self.declare_pattern(pattern, f)
            }

            PatternNode::Name(name) => {
                let name = f(self, name.name);
                self.try_declare_name(name, span);
                Some(name)
            }

            PatternNode::Unit => None,
            PatternNode::Invalid(_) => None,
        }
    }

    /// Try to declare a name, and produce an error message if it already has
    /// been declared.
    fn try_declare_name(&mut self, name: Name, span: Span) {
        if let Some(previous) = self.names.get(&name) {
            self.at(span).duplicate_definition(name, *previous);

            return;
        }

        if let Name::Item(item) = name {
            if let Some(previous) = self.imports.get(&item.name(self.common_db())) {
                self.at(span).duplicate_definition(name, *previous);
            }
        }

        self.names.insert(name, span);
    }

    /// Declare some names in a *declarative scope* nested inside this one.
    fn nested<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let (result, names) = {
            let mut nested = Declarer {
                db: self.db,
                scope: (Vec::new(), self.scope.1),
                names: HashMap::new(),
                imports: HashMap::new(),
            };

            let result = f(&mut nested);
            (result, nested.names)
        };

        for (name, span) in names {
            assert!(
                self.names.insert(name, span).is_none(),
                "TODO: properly scope nested declarative regions"
            );
        }

        result
    }

    /// Declare some names within the scope of another one.
    fn within<F, T>(&mut self, name: DeclarableName, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.scope.0.push(self.scope.1);
        self.scope.1 = name;

        let result = f(self);

        self.scope.1 = self
            .scope
            .0
            .pop()
            .expect("`self.scope` modified outside `self.within()`");
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

use std::collections::{HashMap, HashSet};

use zippy_common::invalid::Reason;
use zippy_common::messages::MessageMaker;
use zippy_common::names::{
    DeclarableName, ItemName, LocalName, Name, RawName, UnnamableName, UnnamableNameKind,
};
use zippy_common::source::{Module, Span};

use crate::messages::NameMessages;
use crate::names::declare::declared_names;
use crate::parser::get_ast;
use crate::{ast, resolved, Db};

#[salsa::tracked]
pub fn resolve_module(db: &dyn Db, module: Module) -> resolved::Module {
    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let name = module.name(zdb);

    // Only referrable names can actually be referred to.
    let declared: HashSet<_> = declared_names(db, module).keys().copied().collect();

    let sources = module.sources(zdb);
    let mut parts = Vec::with_capacity(sources.len());

    for source in sources {
        let ast = get_ast(db, *source);
        let ast_imports = ast.imports(db);
        let ast_items = ast.items(db);

        let imports = imported_names(zdb, DeclarableName::Item(name), ast_imports);
        let mut resolver = PartResolver::new(db, name, &declared, &imports);

        let mut imports = Vec::with_capacity(ast_imports.len());
        let mut items = Vec::with_capacity(ast_items.len());

        for import in ast_imports {
            imports.extend(resolver.resolve_import(import));
        }

        for item in ast_items {
            items.push(resolver.resolve_item(item));
        }

        parts.push(resolved::ModulePart {
            source: *source,
            imports,
            items,
        });
    }

    resolved::Module::new(db, name, parts)
}

/// Get every alias declared by any imports in this module.
fn imported_names(
    db: &dyn zippy_common::Db,
    within: DeclarableName,
    imports: &[ast::Import],
) -> HashMap<ItemName, resolved::Alias> {
    let mut result = HashMap::with_capacity(imports.len());

    for import in imports {
        for name in import.names.iter() {
            let defined = name.span;
            let name = name.alias.name;
            let item_name = ItemName::new(db, Some(within), name);

            // Duplicates are reported by `declared_names`.
            result.insert(item_name, resolved::Alias { name, defined });
        }
    }

    result
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ResolvedName {
    Name(Name),
    Alias(resolved::Alias),
    Neither,
}

struct PartResolver<'a, 'db> {
    db: &'db dyn Db,
    imports: &'a HashMap<ItemName, resolved::Alias>,
    declared: &'a HashSet<Name>,

    parent: (Vec<DeclarableName>, DeclarableName),
    visible_scopes: Vec<usize>,

    outer: Option<&'a PartResolver<'a, 'db>>,
}

impl<'a, 'db> PartResolver<'a, 'db> {
    pub fn new(
        db: &'db dyn Db,
        module: ItemName,
        declared: &'a HashSet<Name>,
        imports: &'a HashMap<ItemName, resolved::Alias>,
    ) -> Self {
        Self {
            db,
            imports,
            declared,
            parent: (Vec::new(), DeclarableName::Item(module)),
            visible_scopes: Vec::new(),
            outer: None,
        }
    }

    pub fn resolve_import(&mut self, import: &ast::Import) -> Option<resolved::Import> {
        let from = self.resolve_expression(match &import.from {
            Some(from) => from,
            None => {
                self.at(import.span).bare_import_unsupported();
                return None;
            }
        });

        let mut names = Vec::with_capacity(import.names.len());
        for name in import.names.iter() {
            let span = name.span;
            let from = name.name;
            let name = ItemName::new(self.common_db(), Some(self.parent.1), name.alias.name);

            let alias = *self
                .imports
                .get(&name)
                .expect("all name aliases must be declared");

            names.push(resolved::ImportedName {
                span,
                name: from,
                alias,
            });
        }

        Some(resolved::Import { from, names })
    }

    pub fn resolve_item(&mut self, item: &ast::Item) -> resolved::Item {
        match item {
            ast::Item::Let {
                pattern,
                anno,
                body,
            } => {
                let pattern_span = pattern.span;
                let (pattern, scope) = self.resolve_pattern(pattern, |resolver, name| {
                    let name = ItemName::new(resolver.common_db(), Some(resolver.parent.1), name);
                    Name::Item(name)
                });

                let scope = match scope {
                    Some(scope) => scope.into(),
                    None => {
                        let name = UnnamableName::new(
                            self.common_db(),
                            UnnamableNameKind::Pattern,
                            Some(self.parent.1),
                            pattern_span,
                        );
                        DeclarableName::Unnamable(name)
                    }
                };

                let (anno, body) = self.within(scope, |resolver| {
                    let anno = anno.as_ref().map(|anno| resolver.resolve_type(anno));
                    let body = body.as_ref().map(|body| resolver.resolve_expression(body));
                    (anno, body)
                });

                resolved::Item::Let {
                    pattern,
                    anno,
                    body,
                }
            }
        }
    }

    fn resolve_type(&mut self, ty: &ast::Type) -> resolved::Type {
        let span = ty.span;
        let node = match &ty.node {
            ast::TypeNode::Range {
                clusivity,
                lower,
                upper,
            } => {
                let lower = self.resolve_expression(lower);
                let upper = self.resolve_expression(upper);
                resolved::TypeNode::Range {
                    clusivity: *clusivity,
                    lower,
                    upper,
                }
            }

            ast::TypeNode::Invalid(reason) => resolved::TypeNode::Invalid(*reason),
        };

        resolved::Type { span, node }
    }

    fn resolve_expression(&mut self, expr: &ast::Expression) -> resolved::Expression {
        let span = expr.span;
        let node = match &expr.node {
            ast::ExpressionNode::Entry { items, imports } => {
                let imported_names = imported_names(self.common_db(), self.parent.1, imports);
                self.nested(&imported_names, |this| {
                    let mut new_items = Vec::new();
                    let mut new_imports = Vec::new();

                    for import in imports {
                        new_imports.extend(this.resolve_import(import));
                    }

                    for item in items {
                        new_items.push(this.resolve_item(item));
                    }

                    resolved::ExpressionNode::Entry {
                        items: new_items,
                        imports: new_imports,
                    }
                })
            }

            ast::ExpressionNode::Block(exprs) => {
                let exprs = exprs
                    .iter()
                    .map(|expr| self.resolve_expression(expr))
                    .collect();
                resolved::ExpressionNode::Block(exprs)
            }

            ast::ExpressionNode::Annotate(expr, ty) => {
                let expr = Box::new(self.resolve_expression(expr));
                let ty = Box::new(self.resolve_type(ty));
                resolved::ExpressionNode::Annotate(expr, ty)
            }

            ast::ExpressionNode::Path(expr, field) => {
                let expr = Box::new(self.resolve_expression(expr));
                resolved::ExpressionNode::Path(expr, *field)
            }

            ast::ExpressionNode::Name(name) => match self.resolve_name(name) {
                ResolvedName::Name(name) => resolved::ExpressionNode::Name(name),
                ResolvedName::Alias(name) => resolved::ExpressionNode::Alias(name),
                ResolvedName::Neither => {
                    self.at(span)
                        .unresolved_name(name.name.text(self.common_db()));
                    resolved::ExpressionNode::Invalid(Reason::NameError)
                }
            },

            ast::ExpressionNode::Number(string) => resolved::ExpressionNode::Number(string.clone()),
            ast::ExpressionNode::String(string) => resolved::ExpressionNode::String(string.clone()),
            ast::ExpressionNode::Unit => resolved::ExpressionNode::Unit,
            ast::ExpressionNode::Invalid(reason) => resolved::ExpressionNode::Invalid(*reason),
        };

        resolved::Expression { span, node }
    }

    fn resolve_pattern<F>(
        &mut self,
        pattern: &ast::Pattern,
        mut f: F,
    ) -> (resolved::Pattern, Option<Name>)
    where
        F: FnMut(&Self, RawName) -> Name,
    {
        let span = pattern.span;
        let (node, name) = match &pattern.node {
            ast::PatternNode::Annotate(pattern, ty) => {
                let (pattern, name) = self.resolve_pattern(pattern, f);
                let pattern = Box::new(pattern);
                let ty = self.resolve_type(ty);
                (resolved::PatternNode::Annotate(pattern, ty), name)
            }

            ast::PatternNode::Name(name) => {
                let name = f(self, name.name);
                (resolved::PatternNode::Name(name), Some(name))
            }

            ast::PatternNode::Unit => (resolved::PatternNode::Unit, None),
            ast::PatternNode::Invalid(reason) => (resolved::PatternNode::Invalid(*reason), None),
        };

        let pattern = resolved::Pattern { span, node };
        (pattern, name)
    }

    fn resolve_name(&self, name: &ast::Identifier) -> ResolvedName {
        // Look for locals
        for scope in self.visible_scopes.iter().rev() {
            let local = LocalName::new(self.common_db(), Some(self.parent.1), name.name, *scope);
            let name = Name::Local(local);
            if self.declared.contains(&name) {
                return ResolvedName::Name(name);
            }
        }

        // Look for items
        let mut parent = Some(self.parent.1);
        loop {
            let item_name = ItemName::new(self.common_db(), parent, name.name);
            let name = Name::Item(item_name);

            if self.declared.contains(&name) {
                return ResolvedName::Name(name);
            }

            if let Some(alias) = self.imports.get(&item_name) {
                return ResolvedName::Alias(*alias);
            }

            if self.common_db().get_module(&item_name).is_some() {
                return ResolvedName::Name(name);
            }

            if let Some(parent_name) = parent {
                parent = parent_name.parent(self.common_db());
            } else {
                break;
            }
        }

        // Look in an outside scope
        self.outer
            .map(|outer| outer.resolve_name(name))
            .unwrap_or(ResolvedName::Neither)
    }

    fn nested<F, T>(&mut self, imports: &HashMap<ItemName, resolved::Alias>, f: F) -> T
    where
        F: for<'b> FnOnce(&mut PartResolver<'b, 'db>) -> T,
    {
        let mut nested = PartResolver {
            db: self.db,
            imports,
            declared: self.declared,
            parent: (Vec::new(), self.parent.1),
            visible_scopes: Vec::new(),
            outer: Some(self),
        };

        f(&mut nested)
    }

    /// Resolve some names within the scope of another one.
    fn within<F, T>(&mut self, name: DeclarableName, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.parent.0.push(self.parent.1);
        self.parent.1 = name;

        let result = f(self);

        self.parent.1 = self
            .parent
            .0
            .pop()
            .expect("`self.scope` modified outside `self.within()`");
        result
    }

    fn at(&self, span: Span) -> MessageMaker<&'db dyn Db> {
        MessageMaker::new(self.db, span)
    }

    fn common_db(&self) -> &'db dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

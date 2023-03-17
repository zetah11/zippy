use std::collections::{HashMap, HashSet};

use zippy_common::invalid::Reason;
use zippy_common::messages::MessageMaker;
use zippy_common::names::{
    DeclarableName, ItemName, LocalName, Name, UnnamableName, UnnamableNameKind,
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
        let mut resolver =
            PartResolver::new(db, DeclarableName::Item(name), &declared, &imports, None);

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

struct PartResolver<'p, 'db> {
    db: &'db dyn Db,
    imports: &'p HashMap<ItemName, resolved::Alias>,
    declared: &'p HashSet<Name>,

    parent: DeclarableName,

    outer: Option<&'p LocaLResolver<'p, 'db>>,
}

impl<'p, 'db> PartResolver<'p, 'db> {
    pub fn new(
        db: &'db dyn Db,
        root: DeclarableName,
        declared: &'p HashSet<Name>,
        imports: &'p HashMap<ItemName, resolved::Alias>,
        outer: Option<&'p LocaLResolver<'p, 'db>>,
    ) -> Self {
        Self {
            db,
            imports,
            declared,
            parent: root,
            outer,
        }
    }

    pub fn resolve_import(&mut self, import: &ast::Import) -> Option<resolved::Import> {
        let Some(from) = &import.from else {
            self.at(import.span).bare_import_unsupported();
            return None;
        };

        let from = self.locals(self.parent, |locals| locals.resolve_expression(from, None));

        let mut names = Vec::with_capacity(import.names.len());
        for name in import.names.iter() {
            let span = name.span;
            let from = name.name;
            let name = ItemName::new(self.common_db(), Some(self.parent), name.alias.name);

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
                let (pattern, name) = self.resolve_pattern(pattern);

                let scope = match name {
                    Some(scope) => DeclarableName::Item(scope),
                    None => {
                        let name = UnnamableName::new(
                            self.common_db(),
                            UnnamableNameKind::Pattern,
                            Some(self.parent),
                            pattern_span,
                        );
                        DeclarableName::Unnamable(name)
                    }
                };

                let (anno, body) = self.locals(scope, |locals| {
                    let anno = anno.as_ref().map(|anno| locals.resolve_type(anno));
                    let body = body
                        .as_ref()
                        .map(|body| locals.resolve_expression(body, name.map(Name::Item)));
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

    fn resolve_pattern(
        &mut self,
        pattern: &ast::Pattern,
    ) -> (resolved::Pattern<ItemName>, Option<ItemName>) {
        let span = pattern.span;
        let (node, name) = match &pattern.node {
            ast::PatternNode::Annotate(pattern, ty) => {
                let (pattern, name) = self.resolve_pattern(pattern);
                let root = match name {
                    Some(root) => DeclarableName::Item(root),
                    None => DeclarableName::Unnamable(UnnamableName::new(
                        self.common_db(),
                        UnnamableNameKind::Pattern,
                        Some(self.parent),
                        span,
                    )),
                };

                let pattern = Box::new(pattern);
                let ty = self.locals(root, |locals| locals.resolve_type(ty));

                (resolved::PatternNode::Annotate(pattern, ty), name)
            }

            ast::PatternNode::Name(name) => {
                let name = ItemName::new(self.common_db(), Some(self.parent), name.name);
                (resolved::PatternNode::Name(name), Some(name))
            }

            ast::PatternNode::Unit => (resolved::PatternNode::Unit, None),
            ast::PatternNode::Invalid(reason) => (resolved::PatternNode::Invalid(*reason), None),
        };

        let pattern = resolved::Pattern { span, node };
        (pattern, name)
    }

    fn resolve_name(&self, name: &ast::Identifier) -> ResolvedName {
        // Look for items
        let mut parent = Some(self.parent);
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

        // Look for locals in an outer scope
        self.outer
            .map(|outer| outer.resolve_name(name))
            .unwrap_or(ResolvedName::Neither)
    }

    fn locals<F, T>(&mut self, root: DeclarableName, f: F) -> T
    where
        F: FnOnce(&mut LocaLResolver) -> T,
    {
        let mut locals = LocaLResolver::new(self.db, self, root);
        f(&mut locals)
    }

    fn at(&self, span: Span) -> MessageMaker<&'db dyn Db> {
        MessageMaker::new(self.db, span)
    }

    fn common_db(&self) -> &'db dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

struct LocaLResolver<'p, 'db> {
    db: &'db dyn Db,
    outer: &'p PartResolver<'p, 'db>,

    parent: DeclarableName,

    scope: usize,
    visible_scopes: Vec<usize>,
}

impl<'p, 'db> LocaLResolver<'p, 'db> {
    pub fn new(db: &'db dyn Db, outer: &'p PartResolver<'p, 'db>, root: DeclarableName) -> Self {
        Self {
            db,
            outer,
            parent: root,

            scope: 0,
            visible_scopes: Vec::new(),
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
                let lower = self.resolve_expression(lower, None);
                let upper = self.resolve_expression(upper, None);
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

    fn resolve_expression(
        &mut self,
        expression: &ast::Expression,
        bind: Option<Name>,
    ) -> resolved::Expression {
        let span = expression.span;
        let node = match &expression.node {
            ast::ExpressionNode::Entry { items, imports } => {
                let root = match bind {
                    Some(bind) => bind.into(),
                    None => DeclarableName::Unnamable(UnnamableName::new(
                        self.common_db(),
                        UnnamableNameKind::Entry,
                        Some(self.parent),
                        span,
                    )),
                };

                let imported_names = imported_names(self.common_db(), self.parent, imports);

                self.items(root, &imported_names, |this| {
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

            ast::ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                let (pattern, bind) = self.resolve_pattern(pattern);
                let pattern = Box::new(pattern);

                let anno = anno
                    .as_ref()
                    .map(|anno| self.resolve_type(anno))
                    .map(Box::new);

                let body = body
                    .as_ref()
                    .map(|body| self.resolve_expression(body, bind.map(Name::Local)))
                    .map(Box::new);

                resolved::ExpressionNode::Let {
                    pattern,
                    anno,
                    body,
                }
            }

            ast::ExpressionNode::Block(exprs, last) => {
                let mut new_exprs = Vec::with_capacity(exprs.len());

                for expression in exprs.iter() {
                    new_exprs.push(self.resolve_expression(expression, None));
                    self.visible_scopes.push(self.scope);
                    self.scope += 1;
                }

                let last = self.resolve_expression(last, None);

                // For every expression, we pushed one visible scope
                self.visible_scopes
                    .truncate(self.visible_scopes.len() - new_exprs.len());

                resolved::ExpressionNode::Block(new_exprs, Box::new(last))
            }

            ast::ExpressionNode::Annotate(expr, ty) => {
                let expr = Box::new(self.resolve_expression(expr, bind));
                let ty = Box::new(self.resolve_type(ty));
                resolved::ExpressionNode::Annotate(expr, ty)
            }

            ast::ExpressionNode::Path(expr, field) => {
                let expr = Box::new(self.resolve_expression(expr, bind));
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

            ast::ExpressionNode::Number(number) => resolved::ExpressionNode::Number(*number),
            ast::ExpressionNode::String(string) => resolved::ExpressionNode::String(*string),
            ast::ExpressionNode::Unit => resolved::ExpressionNode::Unit,
            ast::ExpressionNode::Invalid(reason) => resolved::ExpressionNode::Invalid(*reason),
        };

        resolved::Expression { span, node }
    }

    fn resolve_pattern(
        &mut self,
        pattern: &ast::Pattern,
    ) -> (resolved::Pattern<LocalName>, Option<LocalName>) {
        let span = pattern.span;
        let (node, name) = match &pattern.node {
            ast::PatternNode::Annotate(pattern, ty) => {
                let (pattern, name) = self.resolve_pattern(pattern);
                let pattern = Box::new(pattern);
                let ty = self.resolve_type(ty);
                (resolved::PatternNode::Annotate(pattern, ty), name)
            }

            ast::PatternNode::Name(name) => {
                let name =
                    LocalName::new(self.common_db(), Some(self.parent), name.name, self.scope);
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
            let local = LocalName::new(self.common_db(), Some(self.parent), name.name, *scope);
            let name = Name::Local(local);
            if self.outer.declared.contains(&name) {
                return ResolvedName::Name(name);
            }
        }

        // Look for items
        self.outer.resolve_name(name)
    }

    fn items<F, T>(
        &mut self,
        root: DeclarableName,
        imports: &HashMap<ItemName, resolved::Alias>,
        f: F,
    ) -> T
    where
        F: for<'a> FnOnce(&mut PartResolver<'a, 'db>) -> T,
    {
        let mut resolver =
            PartResolver::new(self.db, root, self.outer.declared, imports, Some(self));
        f(&mut resolver)
    }

    fn at(&self, span: Span) -> MessageMaker<&'db dyn Db> {
        MessageMaker::new(self.db, span)
    }

    fn common_db(&self) -> &dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

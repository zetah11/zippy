use zippy_common::names::ItemName;
use zippy_common::source::Module;

use crate::names::resolve::resolve_module;
use crate::{flattened, resolved, Db};

#[salsa::tracked]
pub fn flatten_module(db: &dyn Db, module: Module) -> flattened::Module {
    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let name = module.name(zdb);
    let resolved = resolve_module(db, module);
    let mut flattener = Flattener::new(name);

    let mut entry = flattened::Entry {
        items: Vec::new(),
        imports: Vec::new(),
    };

    for part in resolved.parts(db) {
        for import in part.imports.iter() {
            entry.imports.push(flattener.flatten_import(import));
        }

        for item in part.items.iter() {
            entry.items.push(flattener.flatten_item(item));
        }
    }

    flattener.builder.build(db, entry)
}

struct Flattener {
    builder: flattened::ModuleBuilder,
}

impl Flattener {
    pub fn new(module_name: ItemName) -> Self {
        Self {
            builder: flattened::ModuleBuilder::new(module_name),
        }
    }

    pub fn flatten_import(&mut self, import: &resolved::Import) -> flattened::ImportIndex {
        let from = self.flatten_expression(&import.from);
        let aliases = import.names.iter().map(|name| name.alias);
        let import = flattened::Import {
            from,
            names: import.names.clone(),
        };

        self.builder.add_import(aliases, import)
    }

    pub fn flatten_item(&mut self, item: &resolved::Item) -> flattened::ItemIndex {
        let (names, item) = match item {
            resolved::Item::Let {
                pattern,
                anno,
                body,
            } => {
                let (pattern, names) = self.flatten_pattern(pattern);
                let anno = anno.as_ref().map(|ty| self.flatten_type(ty));
                let body = body
                    .as_ref()
                    .map(|expression| self.flatten_expression(expression));

                let item = flattened::Item::Let {
                    pattern,
                    anno,
                    body,
                };
                (names, item)
            }
        };

        self.builder.add_item(names, item)
    }

    fn flatten_type(&mut self, ty: &resolved::Type) -> flattened::Type {
        let span = ty.span;
        let node = match &ty.node {
            resolved::TypeNode::Range {
                clusivity,
                lower,
                upper,
            } => {
                let lower = self.flatten_expression(lower);
                let upper = self.flatten_expression(upper);

                let lower = self.builder.add_type_expression(lower);
                let upper = self.builder.add_type_expression(upper);

                flattened::TypeNode::Range {
                    clusivity: *clusivity,
                    lower,
                    upper,
                }
            }

            resolved::TypeNode::Invalid(reason) => flattened::TypeNode::Invalid(*reason),
        };

        flattened::Type { span, node }
    }

    fn flatten_expression(&mut self, expression: &resolved::Expression) -> flattened::Expression {
        let span = expression.span;
        let node = match &expression.node {
            resolved::ExpressionNode::Entry { items, imports } => {
                let mut flat_items = Vec::with_capacity(items.len());
                let mut flat_imports = Vec::with_capacity(imports.len());

                for import in imports {
                    flat_imports.push(self.flatten_import(import));
                }

                for item in items {
                    flat_items.push(self.flatten_item(item));
                }

                let entry = flattened::Entry {
                    items: flat_items,
                    imports: flat_imports,
                };

                flattened::ExpressionNode::Entry(entry)
            }

            resolved::ExpressionNode::Let {
                pattern,
                anno,
                body,
            } => {
                let (pattern, _) = self.flatten_pattern(pattern);
                let anno = anno.as_ref().map(|ty| self.flatten_type(ty));
                let body = body
                    .as_ref()
                    .map(|expression| Box::new(self.flatten_expression(expression)));

                flattened::ExpressionNode::Let {
                    pattern,
                    anno,
                    body,
                }
            }

            resolved::ExpressionNode::Block(expressions) => {
                let expressions = expressions
                    .iter()
                    .map(|expression| self.flatten_expression(expression))
                    .collect();
                flattened::ExpressionNode::Block(expressions)
            }

            resolved::ExpressionNode::Annotate(expression, ty) => {
                let expression = self.flatten_expression(expression);
                let ty = self.flatten_type(ty);
                flattened::ExpressionNode::Annotate(Box::new(expression), ty)
            }

            resolved::ExpressionNode::Path(expression, field) => {
                let expression = self.flatten_expression(expression);
                flattened::ExpressionNode::Path(Box::new(expression), *field)
            }

            resolved::ExpressionNode::Name(name) => flattened::ExpressionNode::Name(*name),
            resolved::ExpressionNode::Alias(alias) => flattened::ExpressionNode::Alias(*alias),
            resolved::ExpressionNode::Number(number) => flattened::ExpressionNode::Number(*number),
            resolved::ExpressionNode::String(string) => flattened::ExpressionNode::String(*string),
            resolved::ExpressionNode::Unit => flattened::ExpressionNode::Unit,

            resolved::ExpressionNode::Invalid(reason) => {
                flattened::ExpressionNode::Invalid(*reason)
            }
        };

        flattened::Expression { span, node }
    }

    fn flatten_pattern<N: Copy>(
        &mut self,
        pattern: &resolved::Pattern<N>,
    ) -> (flattened::Pattern<N>, Vec<N>) {
        let span = pattern.span;
        let (node, names) = match &pattern.node {
            resolved::PatternNode::Annotate(pattern, ty) => {
                let (pattern, names) = self.flatten_pattern(pattern);
                let pattern = Box::new(pattern);
                let ty = self.flatten_type(ty);

                (flattened::PatternNode::Annotate(pattern, ty), names)
            }

            resolved::PatternNode::Name(name) => (flattened::PatternNode::Name(*name), vec![*name]),

            resolved::PatternNode::Unit => (flattened::PatternNode::Unit, Vec::new()),
            resolved::PatternNode::Invalid(reason) => {
                (flattened::PatternNode::Invalid(*reason), Vec::new())
            }
        };

        (flattened::Pattern { span, node }, names)
    }
}

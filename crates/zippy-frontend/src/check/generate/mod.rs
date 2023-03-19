mod check;
mod infer;

use std::collections::HashMap;

use zippy_common::names::Name;
use zippy_common::source::Span;

use super::types::{CoercionVar, Template};
use super::{bound, constrained, Constraint, Type, UnifyVar};
use crate::flattened::{Module, TypeExpression};
use crate::resolved::Alias;

pub struct FlatProgram {
    pub modules: Vec<bound::Module>,
    pub context: HashMap<Name, Template>,
    pub aliases: HashMap<Alias, Type>,
    pub counts: HashMap<Span, usize>,
}

pub struct ConstrainedProgram {
    pub constraints: Vec<Constraint>,
    pub program: constrained::Program,
}

pub fn constrain(program: FlatProgram) -> ConstrainedProgram {
    let mut constrainer = Constrainer::new(program.counts, program.context, program.aliases);

    for module in program.modules {
        constrainer.constrain_module(module);
    }

    constrainer.build()
}

struct Constrainer {
    counts: HashMap<Span, usize>,
    context: HashMap<Name, Template>,
    aliases: HashMap<Alias, Type>,

    constraints: Vec<Constraint>,
    items: constrained::Items,
    imports: constrained::Imports,
    type_exprs: HashMap<(Module, TypeExpression), constrained::Expression>,
}

impl Constrainer {
    pub fn new(
        counts: HashMap<Span, usize>,
        context: HashMap<Name, Template>,
        aliases: HashMap<Alias, Type>,
    ) -> Self {
        Self {
            context,
            aliases,
            counts,

            constraints: Vec::new(),
            items: constrained::Items::new(),
            imports: constrained::Imports::new(),
            type_exprs: HashMap::new(),
        }
    }

    pub fn build(self) -> ConstrainedProgram {
        ConstrainedProgram {
            constraints: self.constraints,
            program: constrained::Program {
                items: self.items,
                imports: self.imports,
                type_exprs: self.type_exprs,
            },
        }
    }

    pub fn constrain_module(&mut self, module: bound::Module) -> constrained::ItemIndex {
        let entry = self.constrain_entry(&module, &module.entry);

        for (name, expression) in module.type_exprs.iter() {
            let expression = self.infer_expr(&module, expression);
            let ty = expression.data.clone();
            self.constraints
                .push(Constraint::TypeNumeric(expression.span, ty));
            assert!(self
                .type_exprs
                .insert((module.module, *name), expression)
                .is_none());
        }

        let ty = module.anno;
        let span = module.span;

        let pattern = constrained::Pattern {
            span,
            data: ty.clone(),
            node: constrained::PatternNode::Name(module.name),
        };

        let body = Some(constrained::Expression {
            span,
            data: ty,
            node: constrained::ExpressionNode::Entry(entry),
        });

        self.items.add(
            std::iter::once(module.name),
            constrained::Item::Let { pattern, body },
        )
    }

    fn constrain_entry(
        &mut self,
        module: &bound::Module,
        entry: &bound::Entry,
    ) -> constrained::Entry {
        let mut names = Vec::new();
        let items = entry
            .items
            .iter()
            .map(|item| {
                names.extend(module.items.names(item));
                self.constrain_item(module, item)
            })
            .collect();

        let imports = entry
            .imports
            .iter()
            .map(|import| self.constrain_import(module, import))
            .collect();

        constrained::Entry {
            imports,
            items,
            names,
        }
    }

    fn constrain_item(
        &mut self,
        module: &bound::Module,
        item: &bound::ItemIndex,
    ) -> constrained::ItemIndex {
        let names = module.items.names(item);
        let item = module.items.get(item);

        let item = match item {
            bound::Item::Let {
                pattern,
                anno,
                body,
            } => {
                let pattern = self.constrain_pattern(pattern);
                let body = body
                    .as_ref()
                    .map(|expression| self.check_expr(module, expression, anno.clone()));
                constrained::Item::Let { pattern, body }
            }
        };

        self.items.add(names, item)
    }

    fn constrain_import(
        &mut self,
        module: &bound::Module,
        import: &bound::ImportIndex,
    ) -> constrained::ImportIndex {
        let import = module.imports.get(import);
        let from = self.infer_expr(module, &import.from);

        let import = constrained::Import {
            from,
            names: import.names.clone(),
        };

        self.imports.add(import)
    }

    fn constrain_pattern<N>(&mut self, pattern: &bound::Pattern<N>) -> constrained::Pattern<N>
    where
        N: Copy + Into<Name>,
    {
        let span = pattern.span;
        let data = pattern.data.clone();
        let node = match &pattern.node {
            bound::PatternNode::Name(name) => constrained::PatternNode::Name(*name),
            bound::PatternNode::Unit => constrained::PatternNode::Unit,
            bound::PatternNode::Invalid(reason) => constrained::PatternNode::Invalid(*reason),
        };

        constrained::Pattern { span, data, node }
    }

    /// Get the instantiated type for the given name.
    fn instantiate(&mut self, at: Span, name: &Name) -> Type {
        let template = self
            .context
            .get(name)
            .expect("all names have been bound")
            .clone();
        let ty = self.fresh(at);
        self.constraints
            .push(Constraint::Instantiated(at, ty.clone(), template));
        ty
    }

    fn fresh(&mut self, at: Span) -> Type {
        let counter = self.counts.entry(at).or_insert(0);
        let count = *counter;
        *counter += 1;

        Type::Var(UnifyVar { span: at, count })
    }

    fn fresh_coercion(&mut self, at: Span) -> CoercionVar {
        let counter = self.counts.entry(at).or_insert(0);
        let count = *counter;
        *counter += 1;

        CoercionVar { span: at, count }
    }
}

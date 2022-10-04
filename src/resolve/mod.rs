// Resolution is admittedly pretty janky because it is unfun to do.

pub mod names;

mod declare_decl;
mod declare_expr;
mod declare_pat;
mod resolve_decl;
mod resolve_expr;
mod resolve_pat;

use crate::hir::{BindId, Decls};
use crate::message::{Messages, Span};
use crate::Driver;
use names::{Actual, Name, Names, Path};

pub fn resolve(driver: &mut impl Driver, decls: Decls) -> (Decls<Name>, Names) {
    let mut resolver = Resolver::new();
    resolver.declare(&decls);
    let decls = resolver.resolve(decls);

    driver.report(resolver.msgs);

    (decls, resolver.names)
}

#[derive(Debug, Default)]
struct Resolver {
    names: Names,
    context: Vec<Name>,
    msgs: Messages,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            names: Names::new(),
            context: Vec::new(),
            msgs: Messages::new(),
        }
    }

    pub fn declare(&mut self, decls: &Decls) {
        self.declare_decls(decls);
        assert!(self.context.is_empty());
    }

    pub fn resolve(&mut self, decls: Decls) -> Decls<Name> {
        self.resolve_decls(decls)
    }

    fn enter(&mut self, span: Span, id: BindId) {
        let path = Path(self.context.clone(), Actual::Scope(id));
        let name = self
            .names
            .lookup(&path)
            .unwrap_or_else(|| self.names.add(span, path));
        self.context.push(name);
    }

    fn exit(&mut self) {
        self.context.pop();
    }

    fn declare_name(&mut self, span: Span, name: Actual) -> Name {
        let path = Path(self.context.clone(), name);

        if let Some(name) = self.names.lookup(&path) {
            let prev = self.names.get_span(&name);
            self.msgs.at(span).resolve_redeclaration(prev);
        }

        self.names.add(span, path)
    }

    fn lookup_name(&mut self, span: Span, name: Actual) -> Option<Name> {
        // lookup strategy; successively remove the last item from the path until we find something
        let mut path = Path(self.context.clone(), name);

        for _ in 0..=path.0.len() {
            if let Some(name) = self.names.lookup(&path) {
                return Some(name);
            }

            path.0.pop();
        }

        self.msgs.at(span).resolve_unknown_name();
        None
    }
}

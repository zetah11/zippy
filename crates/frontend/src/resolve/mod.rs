// Resolution is admittedly pretty janky because it is unfun to do.

mod declare_decl;
mod declare_expr;
mod declare_pat;
mod resolve_decl;
mod resolve_expr;
mod resolve_pat;
mod resolve_type;

use log::{debug, info, trace};

use common::hir::{BindId, Decls};
use common::message::{Messages, Span};
use common::names::{Actual, Name, Names, Path};
use common::Driver;

#[derive(Debug)]
pub struct ResolveRes {
    pub decls: Decls<Name>,
    pub names: Names,
    pub entry: Option<Name>,
}

pub fn resolve(driver: &mut impl Driver, decls: Decls) -> ResolveRes {
    info!("beginning name resolution");
    debug!("declaring");

    let mut resolver = Resolver::new();
    resolver.declare(&decls);

    debug!("resolving");
    let decls = resolver.resolve(decls);

    let entry = driver.entry_name().and_then(|entry| {
        let root = resolver.names.root();
        let res = resolver.names.find_in(root, entry);
        if res.is_none() {
            resolver.msgs.resolve_no_entry_point();
        }
        res
    });

    driver.report(resolver.msgs);

    trace!("done resolving");

    ResolveRes {
        decls,
        names: resolver.names,
        entry,
    }
}

#[derive(Debug)]
struct Resolver {
    names: Names,
    context: (Name, Vec<Name>),
    msgs: Messages,
}

impl Resolver {
    pub fn new() -> Self {
        let mut names = Names::new();
        let context = names.root();
        Self {
            names,
            context: (context, Vec::new()),
            msgs: Messages::new(),
        }
    }

    pub fn declare(&mut self, decls: &Decls) {
        self.declare_decls(decls);
        assert!(self.context.1.is_empty());
    }

    pub fn resolve(&mut self, decls: Decls) -> Decls<Name> {
        self.resolve_decls(decls)
    }

    fn context(&self) -> Name {
        if let Some(top) = self.context.1.last() {
            *top
        } else {
            self.context.0
        }
    }

    fn enter(&mut self, span: Span, id: BindId) {
        let path = Path::new(self.context(), Actual::Scope(id));
        let name = self
            .names
            .lookup(&path)
            .unwrap_or_else(|| self.names.add(span, path));

        self.context.1.push(name);
    }

    fn exit(&mut self) {
        self.context.1.pop();
    }

    fn declare_name(&mut self, span: Span, name: Actual) -> Name {
        let path = Path::new(self.context(), name);

        if let Some(name) = self.names.lookup(&path) {
            let prev = self.names.get_span(&name);
            self.msgs.at(span).resolve_redeclaration(prev);
        }

        self.names.add(span, path)
    }

    fn lookup_name(&mut self, span: Span, name: Actual) -> Option<Name> {
        // lookup strategy; successively remove the last item from the path until we find something
        let mut ctx = self.context();
        let mut path = Path::new(ctx, name);

        loop {
            if let Some(name) = self.names.lookup(&path) {
                return Some(name);
            }

            if let Some(next) = self.names.get_path(&ctx).0 {
                ctx = next;
                path.0 = Some(ctx);
                continue;
            }

            break;
        }

        self.msgs.at(span).resolve_unknown_name();
        None
    }
}

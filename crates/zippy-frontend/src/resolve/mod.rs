mod declare_decl;
mod declare_expr;
mod declare_pat;
mod declare_type;
mod path;
mod resolve_decl;
mod resolve_expr;
mod resolve_pat;
mod resolve_type;

use std::collections::HashMap;

use log::{debug, info};
use zippy_common::{
    message::{Messages, Span},
    names2::{self, Name, NameGenerator},
};

use self::path::{NamePart, Path};
use crate::{resolved, unresolved, Db, MessageAccumulator};

#[salsa::tracked]
pub fn resolve(db: &dyn Db, decls: unresolved::Decls) -> resolved::Decls {
    info!("beginning name resolution");
    debug!("declaring names");

    let mut resolver = Resolver::new(db);
    resolver.declare_decls(&decls);

    debug!("resolving names");

    let decls = resolver.resolve_decls(decls);

    debug!("name resolution done");

    decls
}

pub struct Resolver<'a> {
    /// A mapping from some fully qualified name path to its interned
    /// counterpart.
    names: HashMap<Path, Name>,

    /// The generator used to create new, unique names.
    generator: NameGenerator,

    /// The name of all items we are currently "within", as well as the interned
    /// name of the innermost containing name (if any).
    context: (Vec<NamePart>, Option<Name>),

    db: &'a dyn Db,
}

impl<'a> Resolver<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self {
            names: HashMap::new(),
            generator: NameGenerator::new(),
            context: (Vec::new(), None),

            db,
        }
    }

    /// Add the given name to the current name map (and intern it). Panics if
    /// the name has already been declared.
    fn declare(&mut self, _span: Span, part: NamePart) -> Name {
        let name = match &part {
            NamePart::Scope(_) => self.generator.fresh(self.common_db(), self.context.1),
            NamePart::Source(name) => {
                let text = name.text(self.db);
                Name::new(
                    self.common_db(),
                    self.context.1,
                    names2::NamePart::Source(text.clone()),
                )
            }
        };

        let path = Path(self.context.0.clone(), part);

        assert!(self.names.insert(path, name).is_none());

        name
    }

    /// Run a closure within a scope. The provided closure is given a mutable
    /// reference to a resolver whose context is the given child of the current
    /// context. In contrast to [`Self::in_scope_mut`], this only "resolves" the
    /// scope name, and does not declare it. Consequently, if the scope name
    /// is undeclared, this panics.
    fn in_scope<F, T>(&mut self, part: NamePart, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let path = Path(self.context.0.clone(), part);
        let scope = *self.names.get(&path).expect("undeclared scope");

        let old_context = self.context.1;

        self.context.0.push(part);
        self.context.1 = Some(scope);

        let res = f(self);

        self.context.0.pop();
        self.context.1 = old_context;

        res
    }

    /// Run a closure within a scope. The provided closure is given a mutable
    /// reference to a resolver whose context is the given child of the current
    /// context.
    fn in_scope_mut<F, T>(&mut self, span: Span, part: NamePart, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let scope = self.declare(span, part);
        let old_context = self.context.1;

        self.context.0.push(part);
        self.context.1 = Some(scope);

        let res = f(self);

        self.context.0.pop();
        self.context.1 = old_context;

        res
    }

    /// Lookup a given unqualified name in the current context. Returns `None`
    /// and emits an error message if the name could not be found.
    fn lookup(&self, span: Span, name: unresolved::Name) -> Option<Name> {
        let mut path = Path(self.context.0.clone(), NamePart::Source(name));

        while !path.0.is_empty() {
            if let Some(name) = self.names.get(&path) {
                return Some(*name);
            }

            path.0.pop();
        }

        let result = self.names.get(&path).copied();

        if result.is_none() {
            let name = name.text(self.db);
            self.report_unresolved(span, name);
        }

        result
    }

    fn report_unresolved(&self, span: Span, name: &str) {
        // eww!
        let mut messages = Messages::new();
        messages.at(span).resolve_unknown_name(name);

        for message in messages.msgs {
            MessageAccumulator::push(self.db, message);
        }
    }

    fn common_db(&self) -> &'a dyn zippy_common::Db {
        // oh lord
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

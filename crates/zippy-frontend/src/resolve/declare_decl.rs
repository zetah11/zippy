use super::path::NamePart;
use super::Resolver;
use crate::unresolved::{Decls, TypeDef, ValueDef};

impl Resolver<'_> {
    pub fn declare_decls(&mut self, decls: &Decls) {
        for def in decls.values(self.db) {
            self.declare_value_def(def);
        }

        for def in decls.types(self.db) {
            self.declare_type_def(def);
        }
    }

    fn declare_value_def(&mut self, def: &ValueDef) {
        self.declare_pat(&def.pat);

        self.in_scope_mut(def.span, NamePart::Scope(def.id), |this| {
            for (name, span) in def.implicits.iter().copied() {
                this.declare(span, NamePart::Source(name));
            }

            this.declare_expr(&def.bind);
        });
    }

    fn declare_type_def(&mut self, def: &TypeDef) {
        self.declare_pat(&def.pat);

        self.in_scope_mut(def.span, NamePart::Scope(def.id), |this| {
            this.declare_type(&def.bind);
        });
    }
}

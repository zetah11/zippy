use super::{Name, Resolver};
use crate::hir::{Decls, ValueDef};

impl Resolver {
    pub fn resolve_decls(&mut self, decls: Decls) -> Decls<Name> {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            values.push(self.resolve_value_def(def));
        }

        Decls { values }
    }

    fn resolve_value_def(&mut self, def: ValueDef) -> ValueDef<Name> {
        let pat = self.resolve_pat(def.pat);

        self.enter(def.span, def.id);
        let bind = self.resolve_expr(def.bind);
        self.exit();

        ValueDef {
            span: def.span,
            id: def.id,
            pat,
            anno: def.anno,
            bind,
        }
    }
}

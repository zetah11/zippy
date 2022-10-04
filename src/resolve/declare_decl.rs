use super::Resolver;
use crate::hir::{Decls, ValueDef};

impl Resolver {
    pub fn declare_decls(&mut self, decls: &Decls) {
        for def in decls.values.iter() {
            self.declare_value_def(def);
        }
    }

    fn declare_value_def(&mut self, def: &ValueDef) {
        self.declare_pat(&def.pat);
        self.enter(def.span, def.id);
        self.declare_expr(&def.bind);
        self.exit();
    }
}

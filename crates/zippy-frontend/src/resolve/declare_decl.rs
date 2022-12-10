use zippy_common::hir::{Decls, ValueDef};

use super::{Actual, Resolver};

impl Resolver {
    pub fn declare_decls(&mut self, decls: &Decls) {
        for def in decls.values.iter() {
            self.declare_value_def(def);
        }
    }

    fn declare_value_def(&mut self, def: &ValueDef) {
        self.declare_pat(&def.pat);

        self.enter(def.span, def.id);

        def.implicits.iter().for_each(|(name, span)| {
            self.declare_name(*span, Actual::Lit(name.clone()));
        });

        self.declare_expr(&def.bind);

        self.exit();
    }
}

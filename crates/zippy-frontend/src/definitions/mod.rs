mod bind;

use std::collections::HashMap;

use zippy_common::hir2::{Definitions, Type};
use zippy_common::names2::Name;

use crate::tyck2::lower_type;
use crate::{resolved, Db};

#[salsa::tracked]
pub fn type_definitions(db: &dyn Db, decls: resolved::Decls) -> Definitions {
    let mut definer = Definer::new(db);
    definer.define(decls);

    let db = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    Definitions::new(db, definer.types)
}

struct Definer<'a> {
    db: &'a dyn Db,
    types: HashMap<Name, Type>,
}

impl<'a> Definer<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self {
            db,
            types: HashMap::new(),
        }
    }

    pub fn define(&mut self, decls: resolved::Decls) {
        for def in decls.types(self.db).iter() {
            self.define_typedef(def)
        }
    }

    fn define_typedef(&mut self, def: &resolved::TypeDef) {
        let mut on_wildcard = || {
            // TODO: emit message wildcard not allowed here
            Type::Invalid
        };

        let bind = lower_type(&mut on_wildcard, &def.bind);
        self.bind_type(&def.pat, bind);
    }
}

//! This module is responsible for doing kind inference and -checking. Just like
//! values and variables have types, types and type definitions have kinds. The
//! kind language is significantly simpler than the type language, and as such
//! this module is much simpler than the type checker.
//!
//! TODO: Generalization and instantiation. This will be more relevant once
//! type-level functions are present.

mod bind;
mod infer;
mod kinds;
mod names;
mod pretty;
mod unify;

use std::collections::HashMap;

use zippy_common::message::Messages;
use zippy_common::names2::Name;

use self::kinds::{Kind, UniVar};
use self::names::Namer;
use crate::components::{components, DefIndex};
use crate::resolved::Decls;
use crate::{Db, MessageAccumulator};

#[salsa::tracked]
pub struct Kinds {
    #[return_ref]
    pub kinds: HashMap<Name, zippy_common::kinds::Kind>,
}

#[salsa::tracked]
pub fn kindck(db: &dyn Db, decls: Decls) -> Kinds {
    let mut kinder = Kinder::new(db);
    kinder.infer_decls(decls);

    let mut kinds = HashMap::with_capacity(kinder.context.len());
    let context: Vec<_> = kinder.context.drain().collect();

    for (name, kind) in context {
        kinds.insert(name, kinder.substitute(kind));
    }

    for message in kinder.messages.msgs {
        MessageAccumulator::push(db, message);
    }

    Kinds::new(db, kinds)
}

struct Kinder<'a> {
    db: &'a dyn Db,
    context: HashMap<Name, Kind>,
    subst: HashMap<UniVar, Kind>,

    namer: Namer,
    counter: usize,

    messages: Messages,
}

impl<'a> Kinder<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self {
            db,
            context: HashMap::new(),
            subst: HashMap::new(),

            namer: Namer::new(),
            counter: 0,

            messages: Messages::new(),
        }
    }

    pub fn infer_decls(&mut self, decls: Decls) {
        let types = decls.types(self.db);

        for component in components(self.db, decls).ordered(self.db) {
            let mut type_defs = Vec::new();

            for def in component {
                match def {
                    DefIndex::Type(index) => {
                        type_defs.push(types.get(*index).unwrap());
                    }

                    DefIndex::Value(_) => continue,
                }
            }

            let mut pat_kinds = Vec::with_capacity(type_defs.len());
            for def in type_defs.iter() {
                pat_kinds.push(self.bind(&def.pat));
            }

            for (def, kind) in type_defs.into_iter().zip(pat_kinds) {
                let inferred = self.infer(&def.bind);
                let anno = self.kind_from_type(def.anno.clone());

                self.unify(def.pat.span, kind, inferred.clone());
                self.unify(def.anno.span, anno, inferred);
            }
        }
    }
}

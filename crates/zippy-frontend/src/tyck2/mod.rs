mod bind;
mod check;
mod constrain;
mod infer;
mod lower;
mod unify;

use std::collections::HashMap;

pub use lower::lower_type;

use log::debug;
use zippy_common::hir2::{
    Because, Coercions, Constraint, Context, Decls, Mutability, Type, TypeckResult, UniVar,
    ValueDef,
};
use zippy_common::message::Messages;
use zippy_common::names2::Name;

use crate::components::{components, DefIndex};
use crate::definitions::type_definitions;
use crate::{resolved, Db, MessageAccumulator};

#[salsa::tracked]
pub fn typeck(db: &dyn Db, decls: resolved::Decls) -> TypeckResult {
    let defs = type_definitions(db, decls);

    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let mut typer = Typer::new(db, defs.types(zdb));
    let decls = typer.typeck(decls);

    for message in typer.messages.msgs {
        MessageAccumulator::push(db, message);
    }

    TypeckResult::new(
        zdb,
        typer.coercions,
        typer.context,
        decls,
        typer.subst,
        typer.constraints,
    )
}

struct Typer<'a> {
    // more to come...
    db: &'a dyn Db,
    definitions: &'a HashMap<Name, Type>,

    coercions: Coercions,
    context: Context,
    constraints: Vec<Constraint>,
    subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,

    messages: Messages,
}

impl<'a> Typer<'a> {
    pub fn new(db: &'a dyn Db, definitions: &'a HashMap<Name, Type>) -> Self {
        Self {
            db,
            definitions,

            coercions: Coercions::new(),
            context: Context::new(),
            constraints: Vec::new(),
            subst: HashMap::new(),

            messages: Messages::new(),
        }
    }

    pub fn typeck(&mut self, decls: resolved::Decls) -> Decls {
        let mut values = Vec::new();

        for component in components(self.db, decls).ordered(self.db) {
            debug!("typing a component of {} items", component.len());
            let mut these_values = Vec::new();
            let mut these_types = Vec::new();

            for &index in component {
                match index {
                    DefIndex::Value(index) => {
                        these_values.push(decls.values(self.db).get(index).unwrap())
                    }

                    DefIndex::Type(index) => {
                        these_types.push(decls.types(self.db).get(index).unwrap())
                    }
                }
            }

            // For every strongly connected component of types and values we
            //
            // 1. Bind the value names to their types
            // 2. Check all value definitions
            // 3. Solve any leftover constraints
            //
            // Note that although types can depend upon values (e.g. in
            // `0 upto x`), this dependency relationship never affects the kind
            // of a type. This ultimately means that we can fully check and
            // define all the type names before we check the values they
            // reference.

            // bind values
            // TODO: immutable univars
            let mut bound_values = Vec::new();
            for value in these_values {
                let anno = self.lower_type(&value.anno, Mutability::Mutable);
                let pat = self.bind_pat_schema(&value.pat, anno, &value.implicits);
                bound_values.push((pat, &value.bind, value.span));
            }

            // check values
            for (pat, body, span) in bound_values {
                let body = self.check(Because::Annotation(pat.span), body, pat.data.clone());
                values.push(ValueDef { span, pat, body });
            }

            // solve constraints
            let mut constraint_count = self.constraints.len();
            while constraint_count > 0 {
                let constraints: Vec<_> = self.constraints.drain(..).collect();
                for constraint in constraints {
                    match constraint {
                        Constraint::Assignable {
                            at,
                            id,
                            into,
                            from,
                            subst,
                        } => {
                            self.assign_in(at, subst, id, into, from);
                        }

                        Constraint::Equal {
                            at,
                            t: a,
                            u: b,
                            subst,
                        } => {
                            self.equate_in(at, subst, a, b);
                        }

                        Constraint::NumberType { at, because, ty } => {
                            self.type_number(because, at, ty);
                        }
                    }
                }

                if self.constraints.len() >= constraint_count {
                    for constraint in self.constraints.drain(..) {
                        let span = match constraint {
                            Constraint::Assignable { at, .. } => at,
                            Constraint::Equal { at, .. } => at,
                            Constraint::NumberType { at, .. } => at,
                        };

                        self.messages.at(span).tyck_no_progress();
                    }

                    break;
                }

                constraint_count = self.constraints.len();
            }

            todo!()
        }

        Decls::new(self.common_db(), values)
    }

    fn common_db(&self) -> &'a dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

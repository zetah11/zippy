mod bind;
mod check;
mod constrain;
mod infer;
mod lower;

use log::debug;
use zippy_common::hir2::{Because, Coercions, Constraint, Context, Decls, TypeckResult, ValueDef};
use zippy_common::message::Messages;

use crate::components::{components, DefIndex};
use crate::{resolved, Db, MessageAccumulator};

#[salsa::tracked]
pub fn typeck(db: &dyn Db, decls: resolved::Decls) -> TypeckResult {
    let mut typer = Typer::new(db);
    let decls = typer.typeck(decls);

    for message in typer.messages.msgs {
        MessageAccumulator::push(db, message);
    }

    todo!("type definitions pass")
}

struct Typer<'a> {
    // more to come...
    db: &'a dyn Db,

    coercions: Coercions,
    context: Context,
    constraints: Vec<Constraint>,

    messages: Messages,
}

impl<'a> Typer<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self {
            db,

            coercions: Coercions::new(),
            context: Context::new(),
            constraints: Vec::new(),

            messages: Messages::new(),
        }
    }

    pub fn typeck(&mut self, decls: resolved::Decls) -> Decls {
        let mut values = Vec::new();

        for component in components(self.db, decls).ordered(self.db) {
            debug!("typing a component of {} items", component.len());
            let mut these_values = Vec::new();
            let mut these_types = Vec::new();

            // So this is not great. The basic problem I'm running into is the
            // result of needing some way of associate the keys used in the
            // strongly connected components with the actual elements in the
            // `types` and `values` fields of `decls`. Since, at this stage,
            // these defs may have arbitrarily complicated patterns, we can't
            // just use the name. Using the index is pretty simple, but now we
            // can no longer just `remove` the elements from the vectors, since
            // that might shift the following elements and change their
            // indicies.
            //
            // A potential solution might be to keep track of all the indicies
            // we've removed so far, but (I believe) that would get quadratic in
            // the number of items.
            //
            // There's possibly a very clever solution here using `MaybeUninit`
            // which would let us do this in linear time, but for now, this will
            // have to do.
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
                let anno = self.lower_type(&value.anno);
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
                        Constraint::Assignable { at, into, from, id } => {
                            self.assign_at(at, id, &into, &from);
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

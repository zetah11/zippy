mod bind;
mod check;
mod components;
mod dependency;
mod infer;
mod lower;
mod solve;
mod types;

use log::{debug, info, trace};

use zippy_common::hir2::{
    Because, Constraint, Context, Decls, Expr, ExprNode, Mutability, Pat, PatNode, Type, TypeDef,
    TypeckResult, ValueDef,
};
use zippy_common::message::Messages;
use zippy_common::names2::Name;

use self::components::Components;
use self::dependency::DefIndex;
use self::lower::Lowerer;
use self::solve::Unifier;
use crate::{resolved, Db, MessageAccumulator};

pub fn typeck(db: &dyn Db, decls: resolved::Decls) -> TypeckResult {
    info!("beginning typechecking");

    let (decls, context) = Lowerer::new(db).lower(decls);

    let mut typer = Typer::new(db, context);
    let decls = typer.typeck(decls);

    typer.messages.merge(typer.unifier.messages);

    for msg in typer.messages.msgs {
        MessageAccumulator::push(db, msg);
    }

    trace!("done typechecking");

    TypeckResult::new(
        db,
        typer.unifier.coercions,
        typer.context,
        typer.unifier.defs,
        decls,
        typer.unifier.subst,
        typer.constraints,
    )
}

struct Typer<'a> {
    pub messages: Messages,
    context: Context,
    unifier: Unifier<'a>,
    constraints: Vec<Constraint>,
}

impl<'a> Typer<'a> {
    pub fn new(db: &'a dyn Db, context: Context) -> Self {
        Self {
            messages: Messages::new(),
            context,
            unifier: Unifier::new(db),
            constraints: Vec::new(),
        }
    }

    pub fn typeck(&mut self, decls: resolved::Decls) -> Decls {
        let mut types = Vec::with_capacity(decls.types.len());
        let mut values = Vec::with_capacity(decls.values.len());

        for component in Components::find(&decls) {
            debug!("typing a component of {} items", component.len());
            let mut these_types = Vec::new();
            let mut these_values = Vec::new();

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
            for index in component {
                match index {
                    DefIndex::Type(index) => {
                        these_types.push((*decls.types.get(index).unwrap()).clone())
                    }
                    DefIndex::Value(index) => {
                        these_values.push((*decls.values.get(index).unwrap()).clone())
                    }
                }
            }

            let mut type_pats = Vec::with_capacity(these_types.len());
            let mut val_pats = Vec::with_capacity(these_values.len());

            for def in these_types.iter() {
                self.save_type(&def.pat, &def.bind);
                type_pats.push(self.bind_type_def(def));
            }

            for def in these_values.iter() {
                val_pats.push(self.bind_def(def));
            }

            for (def, pat) in these_types.into_iter().zip(type_pats) {
                let def = self.check_type_def(pat, def);
                types.push(def);
            }

            for (def, pat) in these_values.into_iter().zip(val_pats) {
                let def = self.check_def(pat, def);
                values.push(def);
            }

            debug!("solving {} constraints", self.constraints.len());
            self.solve();
        }

        Decls { values, types }
    }

    fn solve(&mut self) {
        let mut len = self.constraints.len();

        while !self.constraints.is_empty() {
            let constraints: Vec<_> = self.constraints.drain(..).collect();

            for constraint in constraints {
                match constraint {
                    Constraint::IntType { at, because, ty } => {
                        let _ = self.int_type(at, because, ty);
                    }

                    Constraint::Assignable { at, into, from, id } => {
                        self.assignable_coercion(at, into, from, id);
                    }
                }
            }

            if self.constraints.len() >= len {
                match self.constraints.first().unwrap() {
                    Constraint::IntType { at, .. } => self.messages.at(*at).tyck_no_progress(),
                    Constraint::Assignable { at, .. } => self.messages.at(*at).tyck_no_progress(),
                };

                break;
            }

            len = self.constraints.len();
        }
    }

    fn bind_def(&mut self, def: &ValueDef) -> Pat {
        let pat = self.bind_generic(def.pat.clone(), &def.implicits, def.anno.clone());
        self.make_mutability(Mutability::Immutable, &pat);
        pat
    }

    fn bind_type_def(&mut self, def: &resolved::TypeDef) -> Pat {
        let pat = self.bind_pat(def.pat.clone(), def.anno.clone());
        self.make_mutability(Mutability::Immutable, &pat);
        pat
    }

    fn check_def(&mut self, pat: resolved::Pat, def: resolved::ValueDef) -> ValueDef {
        self.make_mutability(Mutability::Mutable, &pat);
        let ty = pat.data.make_mutability(Mutability::Mutable);
        let bind = self.check(Because::Annotation(def.span), def.bind, ty);
        self.make_mutability(Mutability::Immutable, &pat);

        ValueDef {
            pat,
            span: def.span,
            implicits: def.implicits,
            anno: def.anno,
            bind,
        }
    }

    fn check_type_def(&mut self, pat: resolved::Pat, def: resolved::TypeDef) -> TypeDef {
        self.make_mutability(Mutability::Mutable, &pat);
        let ty = pat.data.make_mutability(Mutability::Mutable);
        self.check_type(def.span, &def.bind, ty);
        self.make_mutability(Mutability::Immutable, &pat);

        TypeDef {
            pat,
            span: def.span,
            anno: def.anno,
            bind: def.bind,
        }
    }

    fn make_mutability(&mut self, mutability: Mutability, pat: &Pat) {
        match &pat.node {
            PatNode::Name(name) => self.context.make_mutability(name, mutability),
            PatNode::Anno(pat, _) => self.make_mutability(mutability, pat),
            PatNode::Tuple(x, y) => {
                self.make_mutability(mutability, x);
                self.make_mutability(mutability, y);
            }
            PatNode::Coerce(pat, _) => {
                self.make_mutability(mutability, pat);
            }
            PatNode::Invalid | PatNode::Wildcard => {}
        }
    }

    fn pretty(&mut self, ty: &Type) -> String {
        self.unifier.pretty(ty)
    }
}

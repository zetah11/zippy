use std::collections::HashMap;

use zippy_common::hir2::{Because, Coercion, CoercionId, Constraint, Mutability, Type, UniVar};
use zippy_common::message::Span;
use zippy_common::names2::Name;

use super::unify::{FlowResult, UnificationResult};
use super::Typer;

impl Typer<'_> {
    /// Constrain two types `t` and `u` to be equal.
    pub fn equate(&mut self, span: Span, t: Type, u: Type) {
        self.equate_in(span, HashMap::new(), t, u)
    }

    pub fn equate_in(
        &mut self,
        span: Span,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
        t: Type,
        u: Type,
    ) {
        match self.unify(subst, t, u) {
            UnificationResult::Success { subst } => {
                self.subst.extend(subst);
            }

            UnificationResult::Undecided { unsolved, subst } => {
                for (t, u) in unsolved {
                    self.constraints.push(Constraint::Equal {
                        at: span,
                        t,
                        u,
                        subst: subst.clone(),
                    });
                }
            }

            UnificationResult::Error { occurs, inequal } => {
                for () in occurs {
                    self.messages
                        .at(span)
                        .tyck_recursive_inference("todo!", "todo!!!!");
                }

                for () in inequal {
                    self.messages
                        .at(span)
                        .tyck_incompatible(None::<&str>, None::<&str>);
                }
            }
        }
    }

    /// Constrain the type `from` to be assignable to the type `into` and return
    /// an identifier for the necessary coercion.
    pub fn assign(&mut self, span: Span, into: Type, from: Type) -> CoercionId {
        let id = self.coercions.fresh();
        self.assign_in(span, HashMap::new(), id, into, from);
        id
    }

    pub fn assign_in(
        &mut self,
        span: Span,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
        id: CoercionId,
        into: Type,
        from: Type,
    ) {
        match self.flow(subst, into, from) {
            FlowResult::Success { equal, subst } => {
                if !equal {
                    self.coercions.add(id, Coercion::Upcast);
                }

                self.subst.extend(subst);
            }

            FlowResult::Undecided {
                equal,
                unsolved,
                subst,
            } => {
                if !equal {
                    self.coercions.add(id, Coercion::Upcast);
                }

                for (into, from) in unsolved {
                    self.constraints.push(Constraint::Assignable {
                        at: span,
                        id,
                        into,
                        from,
                        subst: subst.clone(),
                    });
                }
            }

            FlowResult::Error { occurs, inequal } => {
                for () in occurs {
                    self.messages
                        .at(span)
                        .tyck_recursive_inference("todo!!", "todo!!!");
                }

                for () in inequal {
                    self.messages
                        .at(span)
                        .tyck_incompatible(None::<&str>, None::<&str>);
                }
            }
        }
    }

    pub fn type_function(&mut self, because: Because, span: Span, ty: Type) -> (Type, Type) {
        match ty {
            Type::Fun(t, u) => (*t, *u),
            ty @ Type::Var(..) => {
                let t = Type::Var(Mutability::Mutable, self.context.fresh());
                let u = Type::Var(Mutability::Mutable, self.context.fresh());
                self.equate(
                    span,
                    ty,
                    Type::Fun(Box::new(t.clone()), Box::new(u.clone())),
                );

                (t, u)
            }

            _ => {
                // TODO: pretty-print type
                self.messages.at(span).tyck_not_a_fun(None::<&str>);
                (Type::Invalid, Type::Invalid)
            }
        }
    }

    pub fn type_number(&mut self, because: Because, span: Span, ty: Type) -> Type {
        match ty {
            ty @ Type::Range(..) => ty,
            Type::Number => Type::Number,
            Type::Invalid => Type::Invalid,

            ty @ Type::Var(..) => {
                self.constraints.push(Constraint::NumberType {
                    at: span,
                    because,
                    ty: ty.clone(),
                });

                ty
            }

            _ => {
                // TODO: pretty-print type
                self.messages.at(span).tyck_not_an_int(None::<&str>);
                Type::Invalid
            }
        }
    }

    pub fn type_tuple(&mut self, span: Span, ty: Type) -> (Type, Type) {
        match ty {
            Type::Product(t, u) => (*t, *u),
            Type::Invalid => (Type::Invalid, Type::Invalid),

            ty @ Type::Var(..) => {
                let a = Type::Var(Mutability::Mutable, self.context.fresh());
                let b = Type::Var(Mutability::Mutable, self.context.fresh());
                let other = Type::Product(Box::new(a.clone()), Box::new(b.clone()));
                self.equate(span, ty, other);
                (a, b)
            }

            _ => {
                // TODO: pretty-print type
                self.messages.at(span).tyck_tuple_type();
                (Type::Invalid, Type::Invalid)
            }
        }
    }
}

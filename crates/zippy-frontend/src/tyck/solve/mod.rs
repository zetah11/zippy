mod unify;

pub use unify::Unifier;

use std::collections::HashMap;

use log::trace;
use zippy_common::hir2::{merge_insts, Because, CoercionId, Constraint, Type};
use zippy_common::message::Span;
use zippy_common::names2::Name;

use super::Typer;

impl Typer<'_> {
    /// Check if `from` can be given where `into` is expected (i.e. if `from` is wider than `into`), and return the
    /// widest type.
    #[must_use]
    pub fn assignable(&mut self, span: Span, into: Type, from: Type) -> CoercionId {
        trace!(
            "assignability? {} <- {}",
            self.pretty(&into),
            self.pretty(&from)
        );
        let id = self.unifier.coercions.fresh();
        self.assignable_coercion(span, into, from, id);
        id
    }

    pub fn assignable_coercion(&mut self, span: Span, into: Type, from: Type, id: CoercionId) {
        self.unifier.unify(id, span, into, from);
        let constraints: Vec<_> = self
            .unifier
            .worklist
            .drain(..)
            .map(|(at, into, from, id)| Constraint::Assignable { at, into, from, id })
            .collect();
        self.constraints.extend(constraints);
    }

    pub fn fun_type(&mut self, span: Span, ty: Type) -> (Type, Type) {
        let t = self.context.fresh();
        let u = self.context.fresh();
        let expect = Type::Fun(Box::new(Type::mutable(t)), Box::new(Type::mutable(u)));

        // ignore the coercion since we're just unifying with vars
        let _ = self.assignable(span, expect, ty);

        (Type::mutable(t), Type::mutable(u))
    }

    pub fn int_type(&mut self, span: Span, because: Because, ty: Type) -> Type {
        let inst = HashMap::new();
        self.check_int_type(span, because, &inst, ty)
    }

    pub fn tuple_type(&mut self, span: Span, ty: Type) -> (Type, Type) {
        let t = self.context.fresh();
        let u = self.context.fresh();
        let expect = Type::Product(Box::new(Type::mutable(t)), Box::new(Type::mutable(u)));

        let _ = self.assignable(span, expect, ty);

        (Type::mutable(t), Type::mutable(u))
    }

    pub fn hole_type(&mut self, span: Span, ty: Type) -> Type {
        let var = self.context.fresh();
        let _ = self.assignable(span, Type::mutable(var), ty);
        Type::mutable(var)
    }

    fn check_int_type(
        &mut self,
        span: Span,
        because: Because,
        inst: &HashMap<Name, Type>,
        ty: Type,
    ) -> Type {
        match ty {
            Type::Name(name) if inst.contains_key(&name) => {
                let ty = inst.get(&name).unwrap();
                self.check_int_type(span, because, inst, ty.clone())
            }

            Type::Name(name) if self.unifier.defs.has(&name) => {
                let ty = self.unifier.defs.get(&name).unwrap();
                self.check_int_type(span, because, inst, ty.clone())
            }

            Type::Number => Type::Number,
            Type::Range(lo, hi) => Type::Range(lo, hi),
            Type::Invalid => Type::Invalid,

            Type::Instantiated(ty, other_inst) => {
                let inst = merge_insts(inst, &other_inst);
                let ty = self.check_int_type(span, because, &inst, *ty);
                Type::Instantiated(Box::new(ty), inst)
            }

            Type::Var(mutable, v) => {
                if let Some((other_inst, ty)) = self.unifier.subst.get(&v) {
                    let because = if let Some(cause) = self.unifier.causes.get(&v) {
                        cause.clone()
                    } else {
                        because
                    };

                    let inst = merge_insts(inst, other_inst);
                    self.check_int_type(span, because, &inst, ty.clone())
                } else {
                    self.constraints.push(Constraint::IntType {
                        at: span,
                        because,
                        ty,
                    });
                    Type::Var(mutable, v)
                }
            }

            ty => {
                let pretty_type = self.pretty(&ty);
                self.messages.at(span).tyck_not_an_int(Some(pretty_type));
                Type::Invalid
            }
        }
    }
}

use std::collections::HashMap;

use common::names::Name;
pub use unify::Unifier;

mod unify;

use common::message::Span;
use common::thir::{merge_insts, Because, Constraint};

use super::{Type, Typer};

impl Typer<'_> {
    /// Check if `from` can be given where `into` is expected (i.e. if `from` is wider than `into`), and return the
    /// widest type.
    pub fn assignable(&mut self, span: Span, into: Type, from: Type) {
        self.unifier.unify(span, into, from);
        let constraints: Vec<_> = self
            .unifier
            .worklist
            .drain(..)
            .map(|(at, into, from)| Constraint::Assignable { at, into, from })
            .collect();
        self.constraints.extend(constraints);
    }

    pub fn fun_type(&mut self, span: Span, ty: Type) -> (Type, Type) {
        let t = self.context.fresh();
        let u = self.context.fresh();
        let expect = Type::Fun(Box::new(Type::mutable(t)), Box::new(Type::mutable(u)));

        self.assignable(span, expect, ty);

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

        self.assignable(span, expect, ty);

        (Type::mutable(t), Type::mutable(u))
    }

    pub fn hole_type(&mut self, span: Span, ty: Type) -> Type {
        let var = self.context.fresh();
        self.assignable(span, Type::mutable(var), ty);
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

            Type::Range(lo, hi) => Type::Range(lo, hi),
            Type::Invalid => Type::Invalid,

            Type::Instantiated(ty, inst) => self.check_int_type(span, because, &inst, *ty),

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

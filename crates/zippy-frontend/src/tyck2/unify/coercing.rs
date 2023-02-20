use zippy_common::hir2::{merge_insts, Mutability, Type};

use super::{occurs, Inst, Solver};

impl Solver<'_> {
    /// Attempt to find a common substitution of unification variables to types
    /// such that, when applied to both types, `u` can be coerced into `t`.
    pub fn coerce(&mut self, left: &Inst, right: &Inst, into: Type, from: Type) {
        match (into, from) {
            // If there's a definition like `type T = U`, then `U` coerces to `T`
            (Type::Name(n), Type::Name(m)) if n == m => {}
            (Type::Name(n), u) if self.has_definition(&n) => {
                let t = self.get_definition(&n).unwrap().clone();
                self.equal = false;
                self.coerce(left, right, t, u)
            }

            // Any range type can be coerced into any other range type
            // Either runtime checks or post-partial evaluation checks should
            // actually verify that the target type is wider than the source
            // type.
            (Type::Range(lo1, hi1), Type::Range(lo2, hi2)) if lo1 == lo2 && hi1 == hi2 => {}
            (Type::Range(..), Type::Range(..)) => {
                self.equal = false;
            }

            // Any numeric type coerces to the number type
            // TODO: do we actually need to insert coercions here?
            (Type::Number, Type::Range(..)) => {
                self.equal = false;
            }
            (Type::Number, Type::Name(name)) if self.is_numeric(&name) => {
                self.equal = false;
            }

            // Coercing with names from generic instantiations.
            // See the note in [`Self::unify`] for why this is necessary.
            (Type::Name(n), u) if left.contains_key(&n) => {
                let t = left.get(&n).unwrap().clone();
                self.coerce(left, right, t, u)
            }

            (t, Type::Name(m)) if right.contains_key(&m) => {
                let u = right.get(&m).unwrap().clone();
                self.coerce(left, right, t, u)
            }

            // TODO: should functions be coercible? Does allowing this
            // complicate the type system significantly?
            // In principle, we could "switch over" to unifying here, which
            // would probably be fine, but I'm not sure.
            (Type::Fun(t1, u1), Type::Fun(t2, u2)) => {
                // Note that we flip the coercion for the argument type because
                // functions should be contravariant in their argument and
                // covariant in their return type.
                self.coerce(right, left, *t2, *t1);
                self.coerce(left, right, *u1, *u2);
            }

            (Type::Product(t1, u1), Type::Product(t2, u2)) => {
                self.coerce(left, right, *t1, *t2);
                self.coerce(left, right, *u1, *u2);
            }

            // Instantiations merge with the current insts
            (Type::Instantiated(t, inst), u) => {
                let left = merge_insts(left, &inst);
                self.coerce(&left, right, *t, u)
            }

            (t, Type::Instantiated(u, inst)) => {
                let right = merge_insts(right, &inst);
                self.coerce(left, &right, t, *u)
            }

            // Unification vars
            // Note that we don't do any fancy subtyping unification
            // shenanigans here. If we're coercing a type into a univar or vice
            // versa, it's easiest (and most predictable) to say that they are
            // equal (even if doing something more complicated would reject
            // fewer programs).
            (Type::Var(_, v), Type::Var(_, w)) if v == w => {}

            // If a variable has already been solved, coerce with that.
            (Type::Var(mutability, var), u) if self.has(&var) => {
                let (inst, t) = self.get(mutability, &var).unwrap();
                let left = merge_insts(left, inst);
                self.coerce(&left, right, t, u)
            }

            (t, Type::Var(mutability, war)) if self.has(&war) => {
                let (inst, u) = self.get(mutability, &war).unwrap();
                let right = merge_insts(right, inst);
                self.coerce(left, &right, t, u)
            }

            // If a variable is unsolved but mutable, we can substitute it
            // (assuming the variable does not occur on the other side).
            (Type::Var(Mutability::Mutable, var), u) | (u, Type::Var(Mutability::Mutable, var)) => {
                let inst = merge_insts(left, right);
                if occurs(&var, &u) {
                    self.occurs.push(());
                    self.set(inst, var, Type::Invalid);
                } else {
                    self.set(inst, var, u);
                }
            }

            // If a variable is unsolved and immutable, we'll need to solve this
            // later.
            (Type::Var(Mutability::Immutable, var), u)
            | (u, Type::Var(Mutability::Immutable, var)) => {
                let left = if left.is_empty() {
                    Type::immutable(var)
                } else {
                    Type::Instantiated(Box::new(Type::immutable(var)), left.clone())
                };

                let right = if right.is_empty() {
                    u
                } else {
                    Type::Instantiated(Box::new(u), right.clone())
                };

                self.unsolved.push((left, right));
            }

            // Invalid types coerce to and from anything
            (Type::Invalid, _) | (_, Type::Invalid) => {}

            (t, u) => {
                // TODO: pretty print types
                self.inequal.push(());
            }
        }
    }
}

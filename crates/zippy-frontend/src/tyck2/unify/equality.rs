use zippy_common::hir2::{merge_insts, Mutability, Type};

use super::{occurs, Inst, Solver};

impl Solver<'_> {
    pub fn unify(&mut self, left: &Inst, right: &Inst, t: Type, u: Type) {
        match (t, u) {
            // Trivial unifications - `a = a` and the like
            // Note that we don't bother checking for range equality and similar
            // since that's a complicated task for later passes.
            (Type::Name(n), Type::Name(m)) if n == m => {}
            (Type::Range(..), Type::Range(..)) => {}
            (Type::Number, Type::Number) => {}
            (Type::Type, Type::Type) => {}

            // Unifying with names from generic instantiations.
            // This is necessary because the worklist approach may create
            // situations where an unknown type is instantiated and only later
            // revealed to contain one of the instantiated names.
            (Type::Name(n), u) if left.contains_key(&n) => {
                let t = left.get(&n).unwrap().clone();
                self.unify(left, right, t, u)
            }

            (t, Type::Name(m)) if right.contains_key(&m) => {
                let u = right.get(&m).unwrap().clone();
                self.unify(left, right, t, u)
            }

            // Type constructors unify recursively
            (Type::Fun(t1, u1), Type::Fun(t2, u2)) => {
                self.unify(left, right, *t1, *t2);
                self.unify(left, right, *u1, *u2);
            }

            (Type::Product(t1, u1), Type::Product(t2, u2)) => {
                self.unify(left, right, *t1, *t2);
                self.unify(left, right, *u1, *u2);
            }

            // Instantiations merge with the current insts
            (Type::Instantiated(t, inst), u) => {
                let left = merge_insts(left, &inst);
                self.unify(&left, right, *t, u)
            }

            (t, Type::Instantiated(u, inst)) => {
                let right = merge_insts(right, &inst);
                self.unify(left, &right, t, *u)
            }

            // Identical unification vars are equal
            (Type::Var(_, v), Type::Var(_, w)) if v == w => {}

            // If a variable has already been solved, we'll unify with that
            (Type::Var(mutability, var), u) if self.has(&var) => {
                let (inst, t) = self.get(mutability, &var).unwrap();
                let left = merge_insts(left, inst);
                self.unify(&left, right, t, u)
            }

            (t, Type::Var(mutability, war)) if self.has(&war) => {
                let (inst, u) = self.get(mutability, &war).unwrap();
                let right = merge_insts(right, inst);
                self.unify(left, &right, t, u)
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

            // Invalid types unify with anything
            // Note that this is *after* the variable rules to ensure we
            // propagate these.
            (Type::Invalid, _) | (_, Type::Invalid) => {}

            (t, u) => {
                // TODO: pretty print types
                self.inequal.push(());
            }
        }
    }
}

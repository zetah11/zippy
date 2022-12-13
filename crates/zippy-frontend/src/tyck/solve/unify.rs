use std::collections::HashMap;

use log::trace;
use zippy_common::message::{Messages, Span};
use zippy_common::names::{Name, Names};
use zippy_common::thir::{merge_insts, pretty_type, Because, Mutability, PrettyMap, Type, UniVar};

#[derive(Debug)]
pub struct Unifier<'a> {
    pub names: &'a Names,

    pub subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    pub causes: HashMap<UniVar, Because>,
    pub worklist: Vec<(Span, Type, Type)>,
    pub messages: Messages,

    prettier: PrettyMap,
}

impl<'a> Unifier<'a> {
    pub fn new(names: &'a Names) -> Self {
        Self {
            names,
            subst: HashMap::new(),
            causes: HashMap::new(),
            worklist: Vec::new(),
            messages: Messages::new(),

            prettier: PrettyMap::new(),
        }
    }

    pub fn pretty(&mut self, ty: &Type) -> String {
        let subst = self.subst.iter().map(|(var, (_, ty))| (*var, ty)).collect();
        pretty_type(self.names, &subst, &mut self.prettier, ty)
    }

    pub fn unify(&mut self, span: Span, expected: Type, actual: Type) {
        let t = self.pretty(&expected);
        let u = self.pretty(&actual);
        trace!("unifying {t} and {u}",);

        let message_count = self.messages.len();

        self.unify_within(&HashMap::new(), &HashMap::new(), span, expected, actual);

        if self.messages.len() != message_count {
            trace!("unification failure");
        }
    }

    fn unify_within(
        &mut self,
        left_inst: &HashMap<Name, Type>,
        right_inst: &HashMap<Name, Type>,
        span: Span,
        expected: Type,
        actual: Type,
    ) {
        match (expected, actual) {
            (Type::Name(n), u) if left_inst.contains_key(&n) => {
                let t = left_inst.get(&n).unwrap();
                self.unify_within(left_inst, right_inst, span, t.clone(), u)
            }

            (t, Type::Name(m)) if right_inst.contains_key(&m) => {
                let u = right_inst.get(&m).unwrap();
                self.unify_within(left_inst, right_inst, span, t, u.clone())
            }

            (Type::Name(n), Type::Name(m)) => {
                if n != m {
                    let n: Option<String> = Some(self.pretty(&Type::Name(n)));
                    let m: Option<String> = Some(self.pretty(&Type::Name(m)));
                    self.messages.at(span).tyck_incompatible(n, m);
                }
            }

            (Type::Range(lo1, hi1), Type::Range(lo2, hi2)) => {
                todo!()
            }

            (Type::Number, Type::Range(..)) => {}

            (Type::Fun(t1, u1), Type::Fun(t2, u2)) => {
                self.unify_within(left_inst, right_inst, span, *t1, *t2);
                self.unify_within(left_inst, right_inst, span, *u1, *u2);
            }

            (Type::Product(t1, u1), Type::Product(t2, u2)) => {
                self.unify_within(left_inst, right_inst, span, *t1, *t2);
                self.unify_within(left_inst, right_inst, span, *u1, *u2);
            }

            (Type::Var(_, v), Type::Var(_, w)) if v == w => {}

            (Type::Var(mutability, v), u) => {
                if let Some((other_inst, t)) = self.get(mutability, &v) {
                    let left_inst = merge_insts(left_inst, other_inst);
                    return self.unify_within(&left_inst, right_inst, span, t, u);
                }

                if mutability == Mutability::Mutable {
                    let inst = merge_insts(left_inst, right_inst);
                    if Self::occurs(&v, &u) {
                        self.messages
                            .at(span)
                            .tyck_recursive_inference(format!("{v:?}"), format!("{u:?}"));
                        self.set(&inst, v, Type::Invalid);
                    } else {
                        self.set(&inst, v, u);
                        self.causes.insert(v, Because::Unified(span));
                    }
                } else {
                    let left = if left_inst.is_empty() {
                        Type::Var(mutability, v)
                    } else {
                        Type::Instantiated(Box::new(Type::Var(mutability, v)), left_inst.clone())
                    };

                    let right = if right_inst.is_empty() {
                        u
                    } else {
                        Type::Instantiated(Box::new(u), right_inst.clone())
                    };

                    self.worklist.push((span, left, right));
                }
            }

            (t, Type::Var(mutability, w)) => {
                if let Some((other_inst, u)) = self.get(mutability, &w) {
                    let right_inst = merge_insts(right_inst, other_inst);
                    return self.unify_within(left_inst, &right_inst, span, t, u);
                }

                if mutability == Mutability::Mutable {
                    let inst = merge_insts(left_inst, right_inst);
                    if Self::occurs(&w, &t) {
                        self.messages
                            .at(span)
                            .tyck_recursive_inference(format!("{w:?}"), format!("{t:?}"));
                        self.set(&inst, w, Type::Invalid);
                    } else {
                        self.set(&inst, w, t);
                        self.causes.insert(w, Because::Unified(span));
                    }
                } else {
                    let left = if left_inst.is_empty() {
                        t
                    } else {
                        Type::Instantiated(Box::new(t), left_inst.clone())
                    };

                    let right = if right_inst.is_empty() {
                        Type::Var(mutability, w)
                    } else {
                        Type::Instantiated(Box::new(Type::Var(mutability, w)), right_inst.clone())
                    };

                    self.worklist.push((span, left, right));
                }
            }

            (Type::Instantiated(t, other_inst), u) => {
                let new: HashMap<_, _> = left_inst
                    .iter()
                    .chain(other_inst.iter())
                    .map(|(name, var)| (*name, var.clone()))
                    .collect();

                self.unify_within(&new, right_inst, span, *t, u)
            }

            (t, Type::Instantiated(u, other_inst)) => {
                let new: HashMap<_, _> = right_inst
                    .iter()
                    .chain(other_inst.iter())
                    .map(|(name, var)| (*name, var.clone()))
                    .collect();

                self.unify_within(left_inst, &new, span, t, *u)
            }

            (Type::Type, Type::Type) => {}

            (Type::Invalid, _) | (_, Type::Invalid) => {}

            (expected, actual) => {
                let expected = self.pretty(&expected);
                let actual = self.pretty(&actual);

                self.messages
                    .at(span)
                    .tyck_incompatible(Some(expected), Some(actual));
            }
        }
    }

    fn occurs(var: &UniVar, ty: &Type) -> bool {
        match ty {
            Type::Invalid | Type::Number | Type::Type => false,
            Type::Range(_, _) => false,
            Type::Name(_) => false,
            Type::Fun(t, u) => Self::occurs(var, t) || Self::occurs(var, u),
            Type::Product(t, u) => Self::occurs(var, t) || Self::occurs(var, u),
            Type::Instantiated(ty, mapping) => {
                mapping.values().any(|ty| Self::occurs(var, ty)) || Self::occurs(var, ty)
            }
            Type::Var(_, war) => var == war,
        }
    }

    fn get(&self, mutability: Mutability, var: &UniVar) -> Option<(&HashMap<Name, Type>, Type)> {
        self.subst
            .get(var)
            .map(|(inst, ty)| (inst, ty.make_mutability(mutability)))
    }

    fn set(&mut self, inst: &HashMap<Name, Type>, var: UniVar, ty: Type) {
        assert!(self.subst.insert(var, (inst.clone(), ty)).is_none());
    }
}

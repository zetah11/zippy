mod coercing;
mod equality;

use std::collections::HashMap;

use zippy_common::hir2::{Mutability, Type, UniVar};
use zippy_common::names2::Name;

use super::Typer;

pub enum FlowResult {
    Success {
        /// `true` if the two types are equal, `false` if the left-hand side
        /// must be coerced into the right-hand side.
        equal: bool,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    },

    /// A list of unsolved `into <- from` pairs for which we were unable to make
    /// any progress.
    Undecided {
        equal: bool,
        unsolved: Vec<(Type, Type)>,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    },

    Error {
        occurs: Vec<()>,
        inequal: Vec<()>,
    },
}

#[derive(Debug)]
pub enum UnificationResult {
    Success {
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    },

    /// A list of unsolved `t = u` pairs for which we were unable to make any
    /// progress.
    Undecided {
        unsolved: Vec<(Type, Type)>,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    },

    Error {
        occurs: Vec<()>,
        inequal: Vec<()>,
    },
}

impl Typer<'_> {
    /// Attempt to find a substitution from type variables to types which, when
    /// applied to `into` produces a type which is either equal to or can be
    /// coerced into `from`. See [`Self::unify`] for a strict equality check.
    pub fn flow(
        &self,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
        into: Type,
        from: Type,
    ) -> FlowResult {
        let mut solver = Solver::new(self, subst);
        let empty = HashMap::new();
        solver.coerce(&empty, &empty, into, from);

        if !solver.occurs.is_empty() || !solver.inequal.is_empty() {
            FlowResult::Error {
                occurs: solver.occurs,
                inequal: solver.inequal,
            }
        } else if !solver.unsolved.is_empty() {
            FlowResult::Undecided {
                equal: solver.equal,
                unsolved: solver.unsolved,
                subst: solver.subst,
            }
        } else {
            FlowResult::Success {
                equal: solver.equal,
                subst: solver.subst,
            }
        }
    }

    /// Attempt to find a substitution from type variables to types which, when
    /// applied to either, would make `t` and `u` equal. This is a strict form
    /// of equality - a type definition is *not* equal to its body, even though
    /// the latter can be coerced into the former. Use [`Self::flow`] for the
    /// coercive variant of this.
    pub fn unify(
        &self,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
        t: Type,
        u: Type,
    ) -> UnificationResult {
        let mut solver = Solver::new(self, subst);
        let empty = HashMap::new();
        solver.unify(&empty, &empty, t, u);

        if !solver.occurs.is_empty() || !solver.inequal.is_empty() {
            UnificationResult::Error {
                occurs: solver.occurs,
                inequal: solver.inequal,
            }
        } else if !solver.unsolved.is_empty() {
            UnificationResult::Undecided {
                unsolved: solver.unsolved,
                subst: solver.subst,
            }
        } else {
            UnificationResult::Success {
                subst: solver.subst,
            }
        }
    }
}

type Inst = HashMap<Name, Type>;

/// The solver is responsible for solving type equality and subtyping
/// constraints.
struct Solver<'a> {
    typer: &'a Typer<'a>,
    subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    unsolved: Vec<(Type, Type)>,

    /// `true` if the types are equal, `false` if they require a coercion.
    equal: bool,

    /// Occurs-check errors
    occurs: Vec<()>,

    /// Inequal type errors
    inequal: Vec<()>,
}

impl<'a> Solver<'a> {
    pub fn new(typer: &'a Typer<'a>, subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>) -> Self {
        Self {
            typer,
            subst,
            unsolved: Vec::new(),
            equal: true,
            occurs: Vec::new(),
            inequal: Vec::new(),
        }
    }

    fn get(&self, mutability: Mutability, var: &UniVar) -> Option<(&Inst, Type)> {
        self.subst
            .get(var)
            .or_else(|| self.typer.subst.get(var))
            .map(|(inst, ty)| (inst, ty.make_mutability(mutability)))
    }

    fn get_definition(&self, name: &Name) -> Option<&Type> {
        self.typer.definitions.get(name)
    }

    /// Returns true if the given variable has a substition already.
    fn has(&self, var: &UniVar) -> bool {
        self.subst.contains_key(var) || self.typer.subst.contains_key(var)
    }

    fn has_definition(&self, name: &Name) -> bool {
        self.typer.definitions.contains_key(name)
    }

    /// Returns true if the given name is the name of a numeric type.
    fn is_numeric(&self, name: &Name) -> bool {
        match self.typer.definitions.get(name) {
            Some(Type::Name(name)) => self.is_numeric(name),
            Some(Type::Range(..)) => true,
            Some(Type::Invalid) => true,

            Some(Type::Number) => unreachable!(),
            Some(Type::Instantiated(..)) => unreachable!(),
            Some(Type::Var(..)) => unreachable!(),

            Some(_) => false,
            None => false,
        }
    }

    fn set(&mut self, inst: Inst, var: UniVar, ty: Type) {
        assert!(self.subst.insert(var, (inst, ty)).is_none());
    }
}

/// Returns `true` if the given variable occurs anywhere in the given type.
fn occurs(var: &UniVar, ty: &Type) -> bool {
    match ty {
        Type::Name(_) | Type::Range(..) | Type::Number | Type::Type | Type::Invalid => false,
        Type::Fun(t, u) | Type::Product(t, u) => occurs(var, t) || occurs(var, u),

        Type::Instantiated(ty, map) => occurs(var, ty) || map.values().any(|ty| occurs(var, ty)),

        Type::Var(_, war) => var == war,
    }
}

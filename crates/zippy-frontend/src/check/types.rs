use std::collections::HashMap;

use zippy_common::invalid::Reason;
use zippy_common::names::{ItemName, RawName};
use zippy_common::source::Span;

use crate::ast::Clusivity;
use crate::flattened::{Module, TypeExpression};
use crate::resolved::Alias;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Trait { values: HashMap<ItemName, Template> },

    Range(RangeType),

    Unit,
    Number,

    Var(UnifyVar),

    Invalid(Reason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RangeType {
    pub clusivity: Clusivity,
    pub lower: TypeExpression,
    pub upper: TypeExpression,

    /// The module containing the two bounds.
    pub module: Module,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template {
    pub ty: Type,
}

impl Template {
    /// Create a new monomorphic type.
    pub fn mono(ty: Type) -> Self {
        Self { ty }
    }

    /// Create a new template with the same parameters but a different inner
    /// type.
    pub fn with_type(&self, ty: Type) -> Self {
        Self { ty }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    /// Constrain the `alias` to be an alias of the item with the `name` within
    /// some expression of type `of`. The alias should then have a template
    /// equal to that name, that other code can instantiate.
    Alias {
        at: Span,
        alias: Alias,
        of: Type,
        name: RawName,
    },

    /// Constrain `from` to be equal to or coercible to `into`.
    Assignable {
        at: Span,
        id: CoercionVar,
        into: Type,
        from: Type,
    },

    /// Constrain the two given types to be equal.
    Equal(Span, Type, Type),

    /// Constrain `target` to be the equal to the type of the field `field` on
    /// the type `of`.
    Field {
        at: Span,
        target: Type,
        of: Type,
        field: RawName,
    },

    /// Constrain the type to be an instantiation of the given template. This
    /// requires solving the template first, to make sure no type parameters
    /// escape, and *then* instantiating them with unification vars.
    Instantiated(Span, Type, Template),

    /// Constrain the type to be an instantiation of the given imported alias.
    /// This requires solving the type of the import expression first to figure
    /// out what template this even refers to, and then instantiating it.
    InstantiatedAlias(Span, Type, Alias),

    /// Constrain the type to be unit-like.
    UnitLike(Span, Type),

    /// Constrain the type to be numeric.
    Numeric(Span, Type),

    /// Constrain the type to be some string type.
    Textual(Span, Type),

    /// Constrain the type to be numeric, or `number` if it is ambiguous.
    TypeNumeric(Span, Type),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnifyVar {
    pub span: Span,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CoercionVar {
    pub span: Span,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CoercionState {
    Equal,
    Coercible,
    Invalid,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Coercions {
    coercions: HashMap<CoercionVar, CoercionState>,
}

impl Coercions {
    pub fn new() -> Self {
        Self {
            coercions: HashMap::new(),
        }
    }

    pub fn get(&self, var: &CoercionVar) -> CoercionState {
        self.coercions
            .get(var)
            .copied()
            .expect("all coercions have been marked")
    }

    /// Mark the coercion variable with the given state, ensuring no previous
    /// information is lost (e.g. if the variable is marked as `Coercible`
    /// marking it as `Equal` won't change that, but marking it as `Invalid`
    /// will).
    pub fn mark(&mut self, var: CoercionVar, state: CoercionState) {
        if let Some(prev) = self.coercions.get(&var) {
            match (prev, state) {
                (CoercionState::Invalid, _) => return,

                (CoercionState::Coercible, CoercionState::Invalid) => {}
                (CoercionState::Coercible, _) => return,

                (CoercionState::Equal, _) => {}
            }
        }

        self.coercions.insert(var, state);
    }
}

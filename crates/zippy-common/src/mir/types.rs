use super::{Type, TypeId};

/// A memoized store of types. Memoization means that types can be shallowly compared for equality; the two types
/// `0 .. 10 -> 0 .. 10` and `0 .. 10 -> 0 .. 10` will be added bottom-up to this store such that they both end up as
/// `$1` in the store
///
/// ```text
/// $0: 0 .. 10
/// $1: $0 -> $0
/// ```
///
/// where `$n` indicates a type id.
#[derive(Debug, Default)]
pub struct Types {
    types: Vec<Type>,
}

impl Types {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn add(&mut self, ty: Type) -> TypeId {
        if let Some(id) = self.types.iter().position(|other| other == &ty) {
            TypeId(id)
        } else {
            let id = TypeId(self.types.len());
            self.types.push(ty.clone());
            id
        }
    }

    pub fn get(&self, ty: &TypeId) -> &Type {
        self.types.get(ty.0).unwrap()
    }

    pub fn is_function(&self, ty: &TypeId) -> bool {
        matches!(self.get(ty), Type::Fun(..))
    }

    pub fn is_pure(&self, _ty: &TypeId) -> bool {
        true
    }

    pub(super) fn ids(&self) -> impl Iterator<Item = TypeId> {
        (0..self.types.len()).into_iter().map(TypeId)
    }
}

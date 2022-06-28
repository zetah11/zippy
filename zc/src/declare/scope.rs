use std::collections::HashMap;

use crate::name::Name;

/// A unique id to some scope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId(usize);

/// A scope is some section of time within which items live with a common, upper
/// bound on their lifetimes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Scope {
    pub parent: ScopeId,
    pub names: Vec<(String, Name)>,
}

/// A list of scopes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Scopes {
    scopes: HashMap<ScopeId, Scope>,
    current: usize,
}

impl Scopes {
    /// Create an empty list of scopes.
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            current: 0,
        }
    }

    /// Assign a scope to an id. The id can be obtained with
    /// [`Scopes::make_id`].
    pub fn add(&mut self, id: ScopeId, scope: Scope) {
        self.scopes.insert(id, scope);
    }

    /// Get a scope with a given id.
    pub fn get(&self, id: &ScopeId) -> &Scope {
        self.scopes.get(id).unwrap()
    }

    /// Create a unique scope id.
    pub fn make_id(&mut self) -> ScopeId {
        self.current += 1;
        ScopeId(self.current - 1)
    }
}

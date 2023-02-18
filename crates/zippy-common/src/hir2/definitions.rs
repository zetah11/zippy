//! Keeps track of type definitions.

use std::collections::HashMap;

use super::Type;
use crate::names2::Name;

#[salsa::tracked]
pub struct Definitions {
    #[return_ref]
    pub types: HashMap<Name, Type>,
}

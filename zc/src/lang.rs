//! Differentiates between zs and zd. The two languages are identical in terms of parsing and name
//! resolution rules, but differ in certain semantics and type rules.

/// What language is this?
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Language {
    /// zs - the static language, which is AOT compiled and statically typechecks.
    Static,

    /// zd - the dynamic language, which is interpreted and dynamically typechecked.
    Dynamic,
}

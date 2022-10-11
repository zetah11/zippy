use crate::mir::ExprSeq;
use crate::resolve::names::Name;

use super::Env;

#[derive(Clone, Debug)]
pub(super) enum Irreducible {
    Integer(i64),
    Tuple(Vec<Irreducible>),

    Lambda(Name, ExprSeq, Env),
    Quote(ExprSeq),

    Invalid,
}

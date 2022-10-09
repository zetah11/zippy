#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct UniVar(pub(super) usize);

#[derive(Clone, Debug)]
pub enum Type {
    Range(i64, i64),
    Fun(Box<Type>, Box<Type>),

    Product(Box<Type>, Box<Type>),

    Var(UniVar),
    Number,

    Invalid,
}

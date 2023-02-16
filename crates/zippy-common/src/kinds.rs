#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    // Primitive kinds
    Type,

    // Compound kinds
    Function(Box<Kind>, Box<Kind>),
    Product(Box<Kind>, Box<Kind>),

    Invalid,
}

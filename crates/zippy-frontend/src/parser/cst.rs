//! The concrete syntax tree is a very "dumb" representation of the code in a
//! tree form. It's meant to be very accepting, supporting all kinds of
//! constructions which are invalid code. This will be removed and reported
//! during the abstraction pass.

use zippy_common::source::Span;

/// A node in the syntax tree with the span of the area it covers.
#[derive(Clone, Debug)]
pub struct Item {
    pub node: ItemNode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ItemNode {
    /// Brings a name into scope.
    Import(Box<Item>),

    /// A let binding or expression.
    Let {
        pattern: Box<Item>,
        body: Option<Box<Item>>,
    },

    /// A type annotation.
    Annotation(Box<Item>, Box<Item>),

    /// Two items separated by a dot.
    Path(Box<Item>, Box<Item>),

    // Some parenthesized or indented group of delimited items.
    Group(Vec<Item>),

    /// An identifier.
    Name(String),

    /// A numeric literal.
    Number(String),

    /// A string literal.
    String(String),

    /// Some invalid chunk of code and the reason it occured.
    Invalid,
}

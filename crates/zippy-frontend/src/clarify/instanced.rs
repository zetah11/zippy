use zippy_common::invalid::Reason;
use zippy_common::literals::{NumberLiteral, StringLiteral};
use zippy_common::names::{ItemName, LocalName, Name};
use zippy_common::source::Span;

use crate::ast::Identifier;
use crate::checked::{ItemIndex, RangeType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub names: Vec<ItemName>,
    pub node: ItemNode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ItemNode {
    Bound {
        body: Expression,
    },

    Let {
        pattern: Pattern<ItemName>,
        body: Option<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template {
    pub ty: Type,

    /// The existential parameters introduced by this template.
    pub exists: Vec<AbstractInstance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Trait { instance: Instance },

    Range(RangeType),
    Number,
    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub data: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionNode {
    /// The entry dependencies will get filled in at a later pass
    Entry(InstanceIndex),

    Let {
        pattern: Pattern<LocalName>,
        body: Option<Box<Expression>>,
    },

    Block(Vec<Expression>, Box<Expression>),
    Path(Box<Expression>, Identifier),
    Coerce(Box<Expression>),

    Item(ItemIndex),
    Name(Name),
    Number(NumberLiteral),
    String(StringLiteral),

    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern<N> {
    pub node: PatternNode<N>,
    pub data: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternNode<N> {
    Name(N),
    Wildcard,
    Invalid(Reason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InstanceVar(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InstanceIndex(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AbstractInstance(pub(super) usize);

/// A reference to an instance. This may be either an instance parameter, a
/// concrete instance, or a variable to be solved by unification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Instance {
    Concrete(InstanceIndex),
    Parameter(AbstractInstance),
    Var(InstanceVar),
}

/// An entry instance is the actual entry body with an explicit list of
/// dependencies. Entry expressions explicitly supply the dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryInstance {
    pub items: Vec<ItemIndex>,
}

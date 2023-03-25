use std::collections::HashMap;

use zippy_common::invalid::Reason;
use zippy_common::literals::{NumberLiteral, StringLiteral};
use zippy_common::names::{ItemName, LocalName, Name};
use zippy_common::source::Span;

use crate::ast::{Clusivity, Identifier};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Program {
    pub items: HashMap<ItemIndex, Item>,
    pub item_types: HashMap<ItemName, Template>,
    pub local_types: HashMap<LocalName, Type>,
    pub constraints: Vec<Constraint>,

    counter: usize,
}

impl Program {
    pub(crate) fn add_item_for(&mut self, index: ItemIndex, item: Item) {
        assert!(self.items.insert(index, item).is_none());
    }

    pub(crate) fn reserve_item(&mut self) -> ItemIndex {
        let id = ItemIndex(self.counter);
        self.counter += 1;
        id
    }

    pub(crate) fn constrain(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemIndex(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item {
    /// A bound in a number type.
    Bound { body: Expression },

    Let {
        pattern: Pattern<ItemName>,
        body: Option<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    /// Ensure `small` is equal to or a subset of `big`.
    BoundSubset { big: RangeType, small: RangeType },

    /// Ensure the `first` and `second` ranges cover the exact same set of
    /// values.
    BoundEqual { first: RangeType, second: RangeType },

    /// Ensure the given range type contains one or zero values.
    BoundUnitOrEmpty(RangeType),

    /// Ensure the given range type contains exactly one value.
    BoundUnit(RangeType),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template {
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Trait { values: Vec<ItemName> },

    Range(RangeType),

    Number,

    Invalid(Reason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RangeType {
    pub clusivity: Clusivity,
    pub lower: ItemIndex,
    pub upper: ItemIndex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub data: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionNode {
    Entry(Entry),

    Let {
        pattern: Pattern<LocalName>,
        body: Option<Box<Expression>>,
    },

    Block(Vec<Expression>, Box<Expression>),

    // ehhhhh....
    Path(Box<Expression>, Identifier),
    Coerce(Box<Expression>),

    /// A reference to the value bound by an item.
    Item(ItemIndex),
    Name(Name),
    Number(NumberLiteral),
    String(StringLiteral),

    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub items: Vec<ItemIndex>,
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

use std::collections::HashMap;

use zippy_common::invalid::Reason;
use zippy_common::literals::{NumberLiteral, StringLiteral};
use zippy_common::names::{ItemName, LocalName, Name};
use zippy_common::source::Span;

use super::types::{CoercionVar, RangeType, Type};
use crate::ast::Identifier;
use crate::flattened::{Module, TypeExpression};
use crate::resolved::{Alias, ImportedName};

#[derive(Debug)]
pub struct Program {
    pub items: Items,
    pub imports: Imports,
    pub type_exprs: HashMap<(Module, TypeExpression), Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DelayedConstraint {
    /// Ensure the range `small` is equal to or a subset of `big`.
    Subset { big: RangeType, small: RangeType },

    /// Ensure the `first` and `second` ranges cover the exact same set of
    /// values.
    Equal { first: RangeType, second: RangeType },

    /// Ensure the given range type contains one or zero values.
    UnitOrEmpty(RangeType),

    /// Ensure the given range type contains exactly one value.
    Unit(RangeType),
}

#[derive(Debug, Default)]
pub struct Items {
    items: Vec<Item>,
}

impl Items {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: Item) -> ItemIndex {
        let index = ItemIndex(self.items.len());
        self.items.push(item);

        index
    }

    pub fn indicies(&self) -> impl Iterator<Item = ItemIndex> {
        (0..self.items.len()).map(ItemIndex)
    }

    pub fn into_iter(self) -> impl Iterator<Item = (ItemIndex, Item)> {
        self.items
            .into_iter()
            .enumerate()
            .map(|(index, item)| (ItemIndex(index), item))
    }
}

#[derive(Debug, Default)]
pub struct Imports {
    imports: Vec<Import>,
}

impl Imports {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }

    pub fn add(&mut self, import: Import) -> ImportIndex {
        let index = ImportIndex(self.imports.len());
        self.imports.push(import);
        index
    }

    pub fn into_iter(self) -> impl Iterator<Item = Import> {
        self.imports.into_iter()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemIndex(usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImportIndex(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub node: ItemNode,
    pub names: Vec<ItemName>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ItemNode {
    Let {
        pattern: Pattern<ItemName>,
        body: Option<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Import {
    pub from: Expression,
    pub names: Vec<ImportedName>,
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

    Path(Box<Expression>, Identifier),
    Coercion(Box<Expression>, CoercionVar),

    Name(Name),
    Alias(Alias),
    Number(NumberLiteral),
    String(StringLiteral),
    Unit,

    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub items: Vec<ItemIndex>,
    pub imports: Vec<ImportIndex>,
    pub names: Vec<ItemName>,
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
    Unit,
    Invalid(Reason),
}

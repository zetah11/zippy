use std::collections::{HashMap, HashSet};

use zippy_common::invalid::Reason;
use zippy_common::literals::{NumberLiteral, StringLiteral};
use zippy_common::names::{ItemName, LocalName, Name};
use zippy_common::source::Span;

use super::types::Type;
use crate::ast::Identifier;
use crate::flattened::{self, TypeExpression};
use crate::resolved::Alias;

#[derive(Debug, Eq, PartialEq)]
pub struct Module {
    pub module: flattened::Module,
    pub entry: Entry,
    pub items: Items,
    pub imports: Imports,
    pub type_exprs: HashMap<TypeExpression, Expression>,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Items {
    items: Vec<Item>,
    names: HashMap<ItemIndex, HashSet<ItemName>>,
}

impl Items {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            names: HashMap::new(),
        }
    }

    pub fn add(&mut self, names: impl Iterator<Item = ItemName>, item: Item) -> ItemIndex {
        let index = ItemIndex(self.items.len());
        self.items.push(item);
        self.names.insert(index, names.collect());

        index
    }

    pub fn get(&self, index: &ItemIndex) -> &Item {
        self.items
            .get(index.0)
            .expect("index is from this program and always in bounds")
    }

    /// Get every name defined by the given index. This iterator is empty if
    /// the item does not define any names.
    pub fn names(&self, index: &ItemIndex) -> impl Iterator<Item = ItemName> + '_ {
        self.names.get(index).into_iter().flatten().copied()
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
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

    pub fn get(&self, index: &ImportIndex) -> &Import {
        self.imports
            .get(index.0)
            .expect("index is from this program and always in bounds")
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemIndex(usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImportIndex(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item {
    Let {
        pattern: Pattern<ItemName>,
        anno: Type,
        body: Option<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Import {
    pub from: Expression,
    pub names: Vec<ImportedName>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedName {
    pub span: Span,
    pub data: Type,
    pub name: Identifier,
    pub alias: Alias,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionNode {
    Entry(Entry),

    Let {
        pattern: Pattern<LocalName>,
        anno: Type,
        body: Option<Box<Expression>>,
    },

    Block(Vec<Expression>, Box<Expression>),

    Annotate(Box<Expression>, Type),
    Path(Box<Expression>, Identifier),

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

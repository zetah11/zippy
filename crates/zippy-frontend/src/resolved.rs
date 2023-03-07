use zippy_common::invalid::Reason;
use zippy_common::names::{ItemName, LocalName, Name, RawName};
use zippy_common::source::{Source, Span};

use crate::ast::{Clusivity, Identifier};

#[salsa::tracked]
pub struct Module {
    #[id]
    pub name: ItemName,

    #[return_ref]
    pub parts: Vec<ModulePart>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModulePart {
    pub source: Source,
    pub items: Vec<Item>,
    pub imports: Vec<Import>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Import {
    pub from: Expression,
    pub names: Vec<ImportedName>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ImportedName {
    pub span: Span,
    pub name: Identifier,
    pub alias: Alias,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Item {
    Let {
        pattern: Pattern<ItemName>,
        anno: Option<Type>,
        body: Option<Expression>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    Range {
        clusivity: Clusivity,
        lower: Expression,
        upper: Expression,
    },

    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExpressionNode {
    Entry {
        items: Vec<Item>,
        imports: Vec<Import>,
    },

    Block(Vec<Expression>),

    Let {
        pattern: Box<Pattern<LocalName>>,
        anno: Option<Box<Type>>,
        body: Option<Box<Expression>>,
    },

    Annotate(Box<Expression>, Box<Type>),
    Path(Box<Expression>, Identifier),

    Name(Name),
    Alias(Alias),
    Number(String),
    String(String),
    Unit,

    Invalid(Reason),
}

/// A pattern parameterized by the type of the names it may bind.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pattern<N> {
    pub node: PatternNode<N>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PatternNode<N> {
    Annotate(Box<Pattern<N>>, Type),
    Name(N),

    Unit,

    Invalid(Reason),
}

/// An alias is a name that has been imported but not yet resolved. *Eventually*
/// these should resolve to some appropriate [`Name`], but this cannot be done
/// without type information first. An alias is just identified by a name and
/// the span where it is introduced.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Alias {
    pub name: RawName,
    pub defined: Span,
}

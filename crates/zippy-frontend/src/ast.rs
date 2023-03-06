use zippy_common::invalid::Reason;
use zippy_common::names::RawName;
use zippy_common::source::{Source, Span};

#[salsa::tracked]
pub struct AstSource {
    #[id]
    pub source: Source,

    #[return_ref]
    pub items: Vec<Item>,

    #[return_ref]
    pub imports: Vec<Import>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Import {
    pub from: Option<Expression>,
    pub names: Vec<ImportedName>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ImportedName {
    pub span: Span,

    /// The actual name being imported.
    pub name: Identifier,

    /// The name as it appears in this source.
    pub alias: Identifier,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Item {
    Let {
        pattern: Pattern,
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
    /// A collection of items treated as a single object.
    Entry {
        items: Vec<Item>,
        imports: Vec<Import>,
    },

    /// A sequence of expressions delimited by semicolons or newlines.
    Block(Vec<Expression>),

    Annotate(Box<Expression>, Box<Type>),
    Path(Box<Expression>, Identifier),

    Name(Identifier),
    Number(String),
    String(String),
    Unit,

    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pattern {
    pub node: PatternNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PatternNode {
    Annotate(Box<Pattern>, Type),
    Name(Identifier),

    Unit,

    Invalid(Reason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Clusivity {
    pub includes_start: bool,
    pub includes_end: bool,
}

impl Clusivity {
    pub fn exclusive() -> Self {
        Self {
            includes_start: true,
            includes_end: false,
        }
    }

    pub fn inclusive() -> Self {
        Self {
            includes_start: true,
            includes_end: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Identifier {
    pub span: Span,
    pub name: RawName,
}
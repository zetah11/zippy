//! The high-level IR (HIR) is the initial abstract structure for the code after
//! parsing. The HIR is tree-like and represents the structure of the code (i.e.
//! precedence, ordering, etc.) implicitly in the edges of the nodes.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use num_bigint::BigUint;

use crate::source::Span;

/// Abstracts over the different types used in the HIR.
pub trait HirData {
    /// The type of the names of items or variables. Following parsing, this is
    /// a literal string which gets resolved to a simple id during name
    /// resolution.
    type Name: Clone + Debug + Eq + Hash;

    /// The type of the names of a binding. This is distinct from the `Name`
    /// assosciated type in that this may contain some extra information
    /// necessary to store source locations.
    type Binding: Clone + Debug + Eq;

    /// The type a scope introduced by source elements like functions or blocks.
    type Scope: Clone + Debug + Eq;
}

/// A comment preceeding items to document them.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Doc(pub String);

/// Contains all of the declarations in a given declarative section, separated
/// by namespace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decls<D: HirData> {
    /// The scope of these declarations.
    pub scope: D::Scope,
    /// Maps the defined type names with the span of of those names and the type
    /// expression they map to.
    pub types: HashMap<D::Name, TypeDef<D>>,
    /// Maps the defined values with the span of those names, their type
    /// annotation, and the expression they evaluate to.
    pub values: HashMap<D::Name, ValueDef<D>>,
}

impl<D> Default for Decls<D>
where
    D: HirData,
    <D as HirData>::Scope: Default,
{
    fn default() -> Self {
        Self {
            scope: Default::default(),
            types: HashMap::default(),
            values: HashMap::default(),
        }
    }
}

impl<D: HirData> Decls<D> {
    /// Returns true if there are no items declared in this section.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total number of items declared in this section.
    pub fn len(&self) -> usize {
        self.types.len() + self.values.len()
    }
}

/// A type item definition, consisting of its name, the doc comment (if any),
/// and its defining body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeDef<D: HirData> {
    /// The name of this item.
    pub name: D::Binding,
    /// The preceeding documentation for this item.
    pub doc: Option<Doc>,
    /// The definition of this item.
    pub body: Expr<D>,
}

/// A value item definition, consisting of its name, the doc comment (if any),
/// its type annotation, and its defining body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueDef<D: HirData> {
    /// The name of this item.
    pub name: D::Binding,
    /// The preceeding documentation for this item.
    pub doc: Option<Doc>,
    /// The type annotation for this item.
    pub anno: Expr<D>,
    /// The definition of this item.
    pub body: Expr<D>,
}

/// A block is some scope that may contain some declarations and a sequence of
/// statements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block<D: HirData> {
    /// The items declared within this block.
    pub decls: Decls<D>,
    /// The ordered series of statements of this block.
    pub stmts: Stmts<D>,
}

/// A sequence of statements its assosciated scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stmts<D: HirData>(pub Vec<Stmt<D>>, pub D::Scope);

/// A statement is some piece of code that alters the program state in some way,
/// whether that be by changing the control flow or altering memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stmt<D: HirData> {
    /// The actual statement.
    pub node: StmtNode<D>,
    /// The span covering this statement.
    pub span: Span,
}

/// The actual statement. See [`Stmt`] for more.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StmtNode<D: HirData> {
    /// Some scope possibly containing some statements and some declarations.
    Block(Block<D>),

    /// An expression executed purely for its side effects. This is not allowed
    /// to return anything other than a unitary type.
    Expr(Expr<D>),

    /// An `if` statement executes one of its child blocks depending on whether
    /// the condition is true.
    If {
        /// The condition to check.
        cond: Expr<D>,
        /// The code to execute if the condition is true.
        then: Block<D>,
        /// The code to execute if the condition is false.
        elze: Block<D>,
    },

    /// Returns some value from the current function.
    Return(Expr<D>),

    /// Some erroneous statement.
    Invalid,
}

/// An expression computes some value and potentially introduces side-effects.
/// This struct contains the actual [`ExprNode`] as well as its source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expr<D: HirData> {
    /// The actual expression.
    pub node: ExprNode<D>,
    /// The span covering this expression.
    pub span: Span,
}

/// The actual expression. See [`Expr`] for more.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprNode<D: HirData> {
    /// A function definition, and the scope it introduces.
    Fun(Vec<D::Binding>, Block<D>, D::Scope),

    /// A class definition.
    Class(Decls<D>),

    /// A function type `fun(T, U) -> V`.
    Arrow(Vec<Expr<D>>, Box<Expr<D>>),

    /// A function call or operator application.
    Call(Box<Expr<D>>, Vec<Expr<D>>),

    /// Some primitive operator.
    Operator(Operator),

    /// Some string literal
    String(String),

    /// Some regex literal.
    Regex(String),

    /// Some integer literal.
    Integer(BigUint),

    /// Some decimal (floating-point or fixed-point) literal.
    Decimal(String),

    /// Some boolean literal.
    Bool(bool),

    /// The name of an item or variable.
    Name(D::Name),

    /// A wildcard is a "whatever" symbol. In a type context, it signals to the
    /// compiler that the type should be inferred.
    Wildcard,

    /// Some erroneous expression.
    Invalid,
}

/// An operator is a primitve, built-in function. Certain operators like
/// `and do` and `or else` are just syntax sugar for more verbose
/// `if`-expressions, and as such don't have their own operator forms. The
/// remaining operators are in principle built-in functions that can be called
/// like any other, though they are generally overloaded based on type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Operator {
    /// Logical or bitwise `and`, depending on whether the arguments are
    /// booleans or integers.
    And,

    /// Short-circuiting logical `and`. Only accepts boolean arguments, and
    /// cannot be used in higher-order contexts.
    AndDo,

    /// Logical or bitwise `or`, depending on whether the arguments are booleans
    /// or integers.
    Or,

    /// Short-circuiting logical `or`. Only accepts boolean arguments, and
    /// cannot be used in higher-order contexts.
    OrDo,

    /// Logical or bitwise `xor`, depending on whether the arguments are
    /// booleans or integers.
    Xor,

    /// Structural equality `=`
    Equal,

    /// Structural inequality `/=`
    NotEqual,

    /// Less than `<`
    Less,

    /// Less than or equal `<=`
    LessEqual,

    /// Greater than `>`
    Greater,

    /// Greater than or equal `>=`
    GreaterEqual,

    /// Half-open range `upto`
    Upto,

    /// Inclusive range `thru`
    Thru,

    /// Arithmetic addition `+`
    Add,

    /// Arithmetic subtraction `-`
    Subtract,

    /// Arithmetic multiplication `*`
    Multiply,

    /// Arithmetic division `/`
    Divide,

    /// Arithmetic exponent `**`
    Exponent,

    /// Modulus `mod`
    Mod,

    /// Logical or bitwise complement `not`, depending on whether the arguments
    /// are booleans or integers.
    Not,

    /// Arithmetic complement `-`
    Negate,
}

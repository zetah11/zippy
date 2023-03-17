//! This IR represents a "flattened" view of the syntax tree. Whereas earlier
//! representations use a straightforward and easy-to-use encoding of the tree,
//! this one flattens it completely. Every item and expression is stored in a
//! flat list in the module where it is defined, and references to them are done
//! through indicies into that list. This also means that flattened modules give
//! us an easy way of directly looking up any item or expression, no matter how
//! deeply nested, by name.

use std::collections::{HashMap, HashSet};

use zippy_common::invalid::Reason;
use zippy_common::literals::{NumberLiteral, StringLiteral};
use zippy_common::names::{ItemName, LocalName, Name};
use zippy_common::source::Span;

use crate::ast::{Clusivity, Identifier};
use crate::resolved::{Alias, ImportedName};
use crate::Db;

/// Utility struct to safely create [`Module`]s with the correct indicies.
pub(crate) struct ModuleBuilder {
    name: ItemName,

    items: Vec<Item>,
    imports: Vec<Import>,

    item_names: HashMap<ItemIndex, HashSet<ItemName>>,
    expressions: HashMap<TypeExpression, Expression>,
}

impl ModuleBuilder {
    pub fn new(name: ItemName) -> Self {
        Self {
            name,

            items: Vec::new(),
            imports: Vec::new(),

            item_names: HashMap::new(),
            expressions: HashMap::new(),
        }
    }

    pub fn build(self, db: &dyn Db, entry: Entry) -> Module {
        let items = Items {
            items: self.items,
            names: self.item_names,
        };

        let imports = Imports {
            imports: self.imports,
        };

        let type_exprs = TypeExpressions {
            expressions: self.expressions,
        };

        Module::new(db, self.name, entry, items, imports, type_exprs)
    }

    /// Add an item binding all of the specified names to this module. None of
    /// the provided names should have been added before.
    pub fn add_item(&mut self, names: impl IntoIterator<Item = ItemName>, item: Item) -> ItemIndex {
        let index = ItemIndex(self.items.len());
        self.items.push(item);
        self.item_names.insert(index, names.into_iter().collect());

        index
    }

    /// Add an import binding all of the specified names to this module. None of
    /// the provided names should have been added before.
    pub fn add_import(&mut self, import: Import) -> ImportIndex {
        let index = ImportIndex(self.imports.len());
        self.imports.push(import);

        index
    }

    /// Add an expression nested in some expression.
    pub fn add_type_expression(&mut self, expr: Expression) -> TypeExpression {
        let ty = TypeExpression(self.expressions.len());
        self.expressions.insert(ty, expr);
        ty
    }
}

#[salsa::tracked]
pub struct Module {
    /// The name of this module
    #[id]
    pub name: ItemName,

    #[return_ref]
    pub entry: Entry,

    /// Every item declared in this module and any nested entries.
    #[return_ref]
    pub items: Items,

    /// Every import declared in this module and any nested entries.
    #[return_ref]
    pub imports: Imports,

    /// Every expression nested inside a type.
    #[return_ref]
    pub type_exprs: TypeExpressions,
}

impl Module {
    /// Get the item this index refers to.
    pub fn item<'db>(&self, db: &'db dyn Db, index: &ItemIndex) -> &'db Item {
        self.items(db).get(index)
    }

    /// Get the import this index refers to.
    pub fn import<'db>(&self, db: &'db dyn Db, index: &ImportIndex) -> &'db Import {
        self.imports(db).get(index)
    }

    /// Get the expression from the given type expression.
    pub fn type_expression<'db>(&self, db: &'db dyn Db, expr: &TypeExpression) -> &'db Expression {
        self.type_exprs(db).get(expr)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Items {
    items: Vec<Item>,

    /// A mapping from an item index to all the names it defines. This may not
    /// cover all item indicies.
    names: HashMap<ItemIndex, HashSet<ItemName>>,
}

impl Items {
    /// Get the item corresponding to the given index.
    pub fn get(&self, index: &ItemIndex) -> &Item {
        self.items
            .get(index.0)
            .expect("item index is from this module and therefore always in bounds")
    }

    /// Get every name defined by the given index. This iterator is empty if
    /// the item does not define any names.
    pub fn names(&self, index: &ItemIndex) -> impl Iterator<Item = ItemName> + '_ {
        self.names.get(index).into_iter().flatten().copied()
    }

    /// Iterate over every item.
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Imports {
    imports: Vec<Import>,
}

impl Imports {
    /// Get the import corresponding to the given index.
    pub fn get(&self, index: &ImportIndex) -> &Import {
        self.imports
            .get(index.0)
            .expect("import index is from this module and therefore always in bounds")
    }

    /// Iterate over every import.
    pub fn iter(&self) -> impl Iterator<Item = &Import> {
        self.imports.iter()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TypeExpressions {
    expressions: HashMap<TypeExpression, Expression>,
}

impl TypeExpressions {
    pub fn get(&self, expr: &TypeExpression) -> &Expression {
        self.expressions
            .get(expr)
            .expect("type expression is from this module and therefore always in bounds")
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeExpression, &Expression)> {
        self.expressions.iter().map(|(k, v)| (*k, v))
    }
}

/// Represents a kind of reference to some [`Item`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemIndex(usize);

/// Represents a kind of reference to some [`Import`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImportIndex(usize);

/// Represents a kind of reference to an [`Expression`] nested inside a type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeExpression(usize);

/// Represents a list of names imported from the result of a given expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Import {
    pub from: Expression,
    pub names: Vec<ImportedName>,
}

/// An item is some binding in a declarative scope.
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
        lower: TypeExpression,
        upper: TypeExpression,
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
    Entry(Entry),

    Let {
        pattern: Pattern<LocalName>,
        anno: Option<Type>,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Entry {
    /// Every item bound in this entry
    pub items: Vec<ItemIndex>,
    /// Every import bound in this entry
    pub imports: Vec<ImportIndex>,
}

/// A pattern parameterized by the kind of name it can bind.
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

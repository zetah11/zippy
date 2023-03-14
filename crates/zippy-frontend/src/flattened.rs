//! This IR represents a "flattened" view of the syntax tree. Whereas earlier
//! representations use a straightforward and easy-to-use encoding of the tree,
//! this one flattens it completely. Every item and expression is stored in a
//! flat list in the module where it is defined, and references to them are done
//! through indicies into that list. This also means that flattened modules give
//! us an easy way of directly looking up any item or expression, no matter how
//! deeply nested, by name.

use std::collections::HashMap;

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

    item_names: HashMap<ItemName, ItemIndex>,
    import_names: HashMap<Alias, ImportIndex>,
}

impl ModuleBuilder {
    pub fn new(name: ItemName) -> Self {
        Self {
            name,

            items: Vec::new(),
            imports: Vec::new(),

            item_names: HashMap::new(),
            import_names: HashMap::new(),
        }
    }

    pub fn build(self, db: &dyn Db) -> Module {
        let items = Items {
            items: self.items,
            names: self.item_names,
        };

        let imports = Imports {
            imports: self.imports,
            names: self.import_names,
        };

        Module::new(db, self.name, items, imports)
    }

    /// Add an item binding all of the specified names to this module. None of
    /// the provided names should have been added before.
    pub fn add_item<I>(&mut self, names: I, item: Item) -> ItemIndex
    where
        I: IntoIterator<Item = ItemName>,
    {
        let index = ItemIndex(self.items.len());
        self.items.push(item);

        for name in names {
            assert!(self.item_names.insert(name, index).is_none());
        }

        index
    }

    /// Add an import binding all of the specified names to this module. None of
    /// the provided names should have been added before.
    pub fn add_import<I>(&mut self, aliases: I, import: Import) -> ImportIndex
    where
        I: IntoIterator<Item = Alias>,
    {
        let index = ImportIndex(self.imports.len());
        self.imports.push(import);

        for alias in aliases {
            assert!(self.import_names.insert(alias, index).is_none());
        }

        index
    }
}

#[salsa::tracked]
pub struct Module {
    /// The name of this module
    #[id]
    pub name: ItemName,

    /// Every item declared in this module and any nested entries.
    #[return_ref]
    pub items: Items,

    /// Every import declared in this module and any nested entries.
    #[return_ref]
    pub imports: Imports,
}

impl Module {
    /// Get the item where the given name is bound, if it is defined in this
    /// module.
    pub fn item<'db>(&self, db: &'db dyn Db, name: &ItemName) -> Option<&'db Item> {
        self.items(db).get_by_name(name)
    }

    /// Get the item where the given imported alias is bound, if it is defined
    /// in this module.
    pub fn import<'db>(&self, db: &'db dyn Db, alias: &Alias) -> Option<&'db Import> {
        self.imports(db).get_by_name(alias)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Items {
    items: Vec<Item>,

    /// A mapping from every item name to the index of the item in the `items`
    /// field. This split is necessary because each item may bind zero, one or
    /// many names. Thus, the values in this map are not guaranteed to be unique
    /// nor are they guaranteed to cover every index in the array.
    names: HashMap<ItemName, ItemIndex>,
}

impl Items {
    /// Get the item corresponding to the given index.
    pub fn get(&self, index: &ItemIndex) -> &Item {
        self.items
            .get(index.0)
            .expect("item index is from this module and therefore always in bounds")
    }

    /// Get the item corresponding to the given name, if any.
    pub fn get_by_name(&self, name: &ItemName) -> Option<&Item> {
        let index = self.names.get(name)?;
        Some(self.get(index))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Imports {
    imports: Vec<Import>,
    names: HashMap<Alias, ImportIndex>,
}

impl Imports {
    /// Get the import corresponding to the given index.
    pub fn get(&self, index: &ImportIndex) -> &Import {
        self.imports
            .get(index.0)
            .expect("import index is from this module and therefore always in bounds")
    }

    /// Get the import corresponding to the given alias, if any.
    pub fn get_by_name(&self, name: &Alias) -> Option<&Import> {
        let index = self.names.get(name)?;
        Some(self.get(index))
    }
}

/// Represents a kind of reference to some [`Item`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemIndex(usize);

/// Represents a kind of reference to some [`Import`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImportIndex(usize);

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
        /// Every item bound in this entry
        items: Vec<ItemIndex>,

        /// Every import bound in this entry
        imports: Vec<ImportIndex>,
    },

    Let {
        pattern: Box<Pattern<LocalName>>,
        anno: Option<Box<Type>>,
        body: Option<Box<Expression>>,
    },

    Block(Vec<Expression>),

    Annotate(Box<(Expression, Type)>),
    Path(Box<Expression>, Identifier),

    Name(Name),
    Alias(Alias),
    Number(NumberLiteral),
    String(StringLiteral),
    Unit,

    Invalid(Reason),
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

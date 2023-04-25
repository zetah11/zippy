use zippy_common::messages::{Message, MessageContainer, Messages};
use zippy_common::names::Name;
use zippy_common::source::Span;

use crate::Db;

pub trait ClarifyMessages {
    /// Two different, incompatible instances.
    fn incompatible_instances(&mut self);

    /// An instance recursively depends on itself.
    fn recursive_instance(&mut self);
}

pub trait NameMessages {
    /// Bare imports are not yet supported.
    fn bare_import_unsupported(&mut self);

    /// A name has already been defined.
    fn duplicate_definition(&mut self, name: Name, previous: Span);

    /// A name has already been defined as a module.
    fn duplicate_module_definition(&mut self, name: Name);

    /// Name could not be resolved.
    fn unresolved_name(&mut self, name: &str);
}

pub trait ParseMessages {
    /// Expected an expression.
    fn expected_expression(&mut self);

    /// Expected an item.
    fn expected_item(&mut self);

    /// Expected a name.
    fn expected_name(&mut self);

    /// Expected a pattern.
    fn expected_pattern(&mut self);

    /// Expected a type.
    fn expected_type(&mut self);

    /// The indentation is not correct
    fn indent_error(&mut self, expected: usize, actual: usize);

    /// An unmatched parenthesis was encountered.
    fn unclosed_parenthesis(&mut self);

    /// An invalid token was encountered
    fn unexpected_token(&mut self);
}

pub trait TypeMessages {
    /// Types are too unconstrained to be determined.
    fn ambiguous(&mut self);

    /// Cannot unify two inequal types.
    fn inequal_types(&mut self);

    /// The entry is missing a field.
    fn missing_field(&mut self, name: &str);

    /// Cannot get a field from a non-trait type.
    fn not_a_trait(&mut self);

    /// Not a unit type.
    fn not_unitlike(&mut self);

    /// Not a numeric type.
    fn not_numeric(&mut self);

    /// Not a string type.
    fn not_textual(&mut self);

    /// The entry has no such field.
    fn no_such_field(&mut self, name: &str);

    /// A type equation ends up being recursive.
    fn recursive_type(&mut self);
}
impl MessageContainer for &'_ dyn Db {
    fn push(&mut self, message: Message) {
        let db = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(*self);
        Messages::push(db, message)
    }
}

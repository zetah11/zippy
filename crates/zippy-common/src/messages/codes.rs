/// Represents a rough list of every possible "kind" of message. The messages
/// generated by this crate can be very specialized for certain situations, even
/// if they represent what is essentially the same problem, cause, etc. These
/// error codes then serve as a kind of "category" for each message.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Code {
    SyntaxError,
    DeclarationError,
    NameError,
}
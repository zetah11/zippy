mod codes;

pub use self::codes::Code;

use crate::names::Name;
use crate::source::Span;

/// Contains messages produced during queries.
#[salsa::accumulator]
pub struct Messages(Message);

/// A message is some informational message from the compiler to the user about
/// the program. Typically these will be error messages, but they can also be
/// warnings or other kinds of information.
///
/// Messages can be ergonomically created using [`MessageMaker`] and the
/// [`MessageContainer`] trait.

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Message {
    /// A short, descriptive title of this message. Ideally, position
    /// information (from the span) and this title should be enough information
    /// for the programmer to act on the message.
    pub title: Text,

    /// A code indicating the cause of this message.
    pub code: Code,

    /// The severity level of this message. Errors typically mean there is no
    /// unambiguous semantic interpretation of the program. Warnings indicate
    /// possible incorrectness which otherwise has a semantic meaning. Info
    /// refers to other kinds of informatino.
    pub severity: Severity,

    /// The place where this message occurred.
    pub span: Span,

    /// Additional locations in the source code with some helpful information.
    pub labels: Vec<(Span, Text)>,

    /// Extra information not associated with a particular place in the code,
    /// such as more detailed information or suggestions.
    pub notes: Vec<(NoteKind, Text)>,
}

impl Message {
    pub fn error(code: Code, span: Span) -> Self {
        Self {
            span,
            code,
            severity: Severity::Error,

            title: Text::empty(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn warning(code: Code, span: Span) -> Self {
        Self {
            span,
            code,
            severity: Severity::Warning,

            title: Text::empty(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn info(code: Code, span: Span) -> Self {
        Self {
            span,
            code,
            severity: Severity::Info,

            title: Text::empty(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_title(self, title: Text) -> Self {
        Self { title, ..self }
    }

    pub fn with_labels(self, labels: Vec<(Span, Text)>) -> Self {
        Self { labels, ..self }
    }

    pub fn with_notes(self, notes: Vec<(NoteKind, Text)>) -> Self {
        Self { notes, ..self }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NoteKind {
    Note,
    Help,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Some rich text.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Text(pub Vec<TextPart>);

impl Text {
    /// Some empty text.
    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

/// A single chunk of richly formatted text.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TextPart {
    /// Plain text.
    Text(String),

    /// Some piece of code.
    Code(String),

    /// Some user-defined name.
    Name(Name),
}

/// This is used as an intermediate struct to generate messages using syntax
/// like `self.at(span).my_message_method()`. Particular methods can be
/// implemented as extension traits on this structure, and it takes care of
/// adding it to the right container (such as a list of messages or a database).
pub struct MessageMaker<C> {
    pub span: Span,
    container: C,
}

impl<C> MessageMaker<C> {
    pub fn new(container: C, span: Span) -> Self {
        Self { span, container }
    }

    pub fn error(&self, code: Code) -> Message {
        Message::error(code, self.span)
    }

    pub fn warning(&self, code: Code) -> Message {
        Message::warning(code, self.span)
    }

    pub fn info(&self, code: Code) -> Message {
        Message::info(code, self.span)
    }
}

impl<C: MessageContainer> MessageMaker<C> {
    pub fn add(&mut self, message: Message) {
        C::push(&mut self.container, message);
    }
}

/// This trait represents anything which is able to accumulate a bunch of
/// messages at once. For instance, `&dyn Db` might implement this using the
/// [`Messages`] accumulator.
pub trait MessageContainer {
    fn push(&mut self, message: Message);
}

impl MessageContainer for &'_ dyn crate::Db {
    fn push(&mut self, message: Message) {
        Messages::push(*self, message)
    }
}

impl MessageContainer for &'_ mut Vec<Message> {
    fn push(&mut self, message: Message) {
        Vec::push(*self, message)
    }
}

/// Helper macro to generate some rich [`Text`].
///
/// ```
/// let _ = zippy_common::text![
///     "here is some ",
///     (code "code"),
///     " for you"
/// ];
/// ```
#[macro_export]
macro_rules! text {
    (part (code $part:tt)) => {
        $crate::messages::TextPart::Code($part.into())
    };

    (part (name $part:tt)) => {
        $crate::messages::TextPart::Name($part)
    };

    (part $part:tt) => {
        $crate::messages::TextPart::Text($part.into())
    };

    [ $( $part:tt ),* ] => {
        $crate::messages::Text(vec![
            $( $crate::text!(part $part) ),*
        ])
    };
}

//! Errors and messages are the primary means the compiler communicates to the
//! programmer with.

use crate::source::Span;

/// Represents some informational message like a warning or an error.
#[derive(Clone, Debug, Eq)]
pub struct Message {
    /// How severe is this message?
    pub severity: Severity,

    /// Some integer code that identifies this particular kind of message.
    pub code: u32,

    /// A short description that summarizes the message.
    pub title: String,

    /// The main message text.
    pub message: String,

    /// The main location that this message points to.
    pub at: Span,

    /// Additional labels assosciated with specific locations in this code.
    pub labels: Vec<(String, Span)>,

    /// Additional messages that aren't tied to any specific location.
    pub notes: Vec<String>,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.at == other.at
    }
}

/// Differentiates between different levels of severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    /// Indicates a possible mistake which still allows the program to
    /// successfully compile.
    Warning,

    /// Indicates a possible mistake which prevents successful interpretation or
    /// compilation of the program.
    Error,
}

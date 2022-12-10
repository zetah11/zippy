use std::cmp::Ordering;

use super::Span;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity) -> Self {
        Self {
            severity,
            code: None,
            message: String::new(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn bug() -> Self {
        Self::new(Severity::Bug)
    }

    pub fn error() -> Self {
        Self::new(Severity::Error)
    }

    pub fn warning() -> Self {
        Self::new(Severity::Warning)
    }

    pub fn note() -> Self {
        Self::new(Severity::Note)
    }

    pub fn help() -> Self {
        Self::new(Severity::Help)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_labels(mut self, labels: Vec<Label>) -> Self {
        self.labels.extend(labels);
        self
    }

    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes.extend(notes.into_iter().map(Into::into));
        self
    }
}

#[derive(Clone, Debug)]
pub struct Label {
    pub style: LabelStyle,
    pub span: Span,
    pub message: String,
}

impl Label {
    pub fn new(style: LabelStyle, span: Span) -> Self {
        Self {
            style,
            span,
            message: String::new(),
        }
    }

    pub fn primary(span: Span) -> Self {
        Self::new(LabelStyle::Primary, span)
    }

    pub fn secondary(span: Span) -> Self {
        Self::new(LabelStyle::Secondary, span)
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Severity {
    Bug,
    Error,
    Warning,
    Note,
    Help,
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Bug, Self::Bug) => Ordering::Equal,
            (Self::Bug, _) => Ordering::Greater,
            (_, Self::Bug) => Ordering::Less,

            (Self::Error, Self::Error) => Ordering::Equal,
            (Self::Error, _) => Ordering::Greater,
            (_, Self::Error) => Ordering::Less,

            (Self::Warning, Self::Warning) => Ordering::Equal,
            (Self::Warning, _) => Ordering::Greater,
            (_, Self::Warning) => Ordering::Less,

            (Self::Note, Self::Note) => Ordering::Equal,
            (Self::Note, _) => Ordering::Greater,
            (_, Self::Note) => Ordering::Less,

            (Self::Help, Self::Help) => Ordering::Equal,
        }
    }
}

use std::convert::Infallible;

use lalrpop_util::{ErrorRecovery, ParseError};

use crate::lex::Token;
use crate::message::{Message, Severity};
use crate::source::SourceId;

/// Convert a LALRPOP-specific `ErrorRecovery` into the zc message type.
pub fn to_message(err: ErrorRecovery<usize, Token, Infallible>, at: SourceId) -> Message {
    let _dropped = err
        .dropped_tokens
        .into_iter()
        .map(|(start, tok, end)| (tok, at.span(start, end)))
        .collect::<Vec<_>>();

    match err.error {
        ParseError::User { error } => match error {},
        ParseError::InvalidToken { location } => Message {
            severity: Severity::Error,
            at: at.span(location, location),
            code: 10,
            title: "unexpected token".into(),
            message: "unexpected token or end-of-file".into(),
            labels: vec![],
            notes: vec![],
        },
        ParseError::UnrecognizedEOF { location, expected } => Message {
            severity: Severity::Error,
            at: at.span(location, location),
            code: 11,
            title: "unexpected end-of-file".into(),
            message: format!(
                "expected {}, but reached end of file",
                expected.into_iter().take(5).collect::<Vec<_>>().join(", ")
            ),
            labels: vec![],
            notes: vec![],
        },
        ParseError::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        } => Message {
            severity: Severity::Error,
            at: at.span(start, end),
            code: 10,
            title: format!("unexpected token '{}'", tok),
            message: format!(
                "expected {}",
                expected.into_iter().take(5).collect::<Vec<_>>().join(", ")
            ),
            labels: vec![],
            notes: vec![],
        },
        ParseError::ExtraToken {
            token: (start, tok, end),
        } => Message {
            severity: Severity::Error,
            at: at.span(start, end),
            code: 10,
            message: "unexpected token".into(),
            title: format!("unexpected token '{}'", tok),
            labels: vec![],
            notes: vec![],
        },
    }
}

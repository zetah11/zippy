use lsp_document::{IndexedText, Pos, TextAdapter, TextMap};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
    Position, Range, Url,
};
use zippy_common::messages::{Message, Severity};
use zippy_common::source::Span;

use super::format::format_text;
use super::Backend;
use crate::database::Database;
use crate::meta;
use crate::output::format_code;
use crate::pretty::Prettier;

impl Backend {
    /// Create an LSP diagnostic for the given message, as well as the URI for
    /// the source it comes from.
    pub fn make_diagnostic(&self, prettier: &Prettier, message: Message) -> (Url, Diagnostic) {
        let severity = match message.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        };

        let span = message.span;

        let range = span_to_range(&self.database, span);
        let url = span_to_uri(&self.database, span);

        let related_information = message
            .labels
            .into_iter()
            .flat_map(|(span, label)| {
                let message = format_text(prettier, label);
                let location = Location {
                    range: span_to_range(&self.database, span),
                    uri: span_to_uri(&self.database, span),
                };

                Some(DiagnosticRelatedInformation { location, message })
            })
            .collect();

        let code = format_code(message.code);
        let message = format_text(prettier, message.title);

        let diagnostic = Diagnostic {
            range,
            severity: Some(severity),
            code: Some(NumberOrString::String(code.into())),
            code_description: None,
            source: Some(meta::COMPILER_NAME.into()),
            message,
            related_information: Some(related_information),
            tags: None,
            data: None,
        };

        (url, diagnostic)
    }
}

/// Get the URI for the given span.
fn span_to_uri(db: &Database, span: Span) -> Url {
    let path = span.source.name(db);
    let path = db.get_source_path(path);

    match Url::from_file_path(path) {
        Ok(url) => url,
        Err(()) => panic!("{} is not a file", path.display()),
    }
}

/// Convert the given span to an LSP range.
fn span_to_range(db: &Database, span: Span) -> Range {
    let indexed = IndexedText::new(span.source.content(db).as_ref());
    let range = indexed
        .range_to_lsp_range(
            &indexed
                .offset_range_to_range(span.start..span.end)
                .unwrap_or(Pos::new(0, 0)..Pos::new(0, 0)),
        )
        .unwrap_or_default();

    Range {
        start: Position {
            line: range.start.line,
            character: range.start.character,
        },
        end: Position {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

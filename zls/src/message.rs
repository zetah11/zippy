use ropey::Rope;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use zc::message::{Message, Severity};

pub fn to_diagnostic(msg: &Message, rope: &Rope) -> Option<Diagnostic> {
    let start = offset_to_position(msg.at.start, rope)?;
    let end = offset_to_position(msg.at.end, rope)?;
    let severity = match msg.severity {
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Error => DiagnosticSeverity::ERROR,
    };

    Some(Diagnostic::new_with_code_number(
        Range::new(start, end),
        severity,
        i32::try_from(msg.code).ok()?,
        Some("zc".into()),
        msg.message.clone(),
    ))
}

fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char;
    Some(Position::new(line as u32, column as u32))
}

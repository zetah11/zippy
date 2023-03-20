use zippy_common::messages::{Code, NoteKind};

/// Get a nice human-readable version of an error code.
pub fn format_code(code: Code) -> &'static str {
    match code {
        Code::SyntaxError => "syntax error",
        Code::DeclarationError => "declaration error",
        Code::NameError => "name error",
        Code::TypeError => "type error",
    }
}

/// Get a nice human-readable version of a note kind.
pub fn format_note_kind(kind: NoteKind) -> &'static str {
    match kind {
        NoteKind::Note => "note",
        NoteKind::Help => "help",
    }
}

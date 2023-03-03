use zippy_common::messages::Code;

/// Get a nice human-readable version of an error code.
pub fn format_code(code: Code) -> &'static str {
    match code {
        Code::SyntaxError => "syntax error",
    }
}

use zippy_common::source::Span;

pub struct SpanStartInfo {
    /// The number of linefeeds (`\n`) before the span start.
    pub line: usize,

    /// The byte index of the first character in the line that contains the span
    /// start.
    pub line_index: usize,

    /// Number of `char`s before the span start on the line.
    pub column: usize,
}

/// Compute some information about where this span starts.
///
/// # Panics
///
/// Panics if the give span is outside the text.
pub fn find_span_start(text: &str, span: Span) -> SpanStartInfo {
    let mut line = 0;
    let mut line_index = 0;
    let mut column = 0;
    let mut inside = false;

    for (i, c) in text.char_indices() {
        if i >= span.start {
            column -= 1;
            inside = true;
            break;
        }

        if c == '\n' {
            line += 1;
            line_index = i + 1; // newlines are one byte long, this is okay
            column = 0;
            continue;
        }

        column += 1;
    }

    assert!(inside);

    SpanStartInfo {
        line,
        line_index,
        column,
    }
}

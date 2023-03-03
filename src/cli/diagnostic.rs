use crossterm::terminal;
use zippy_common::messages::{Message, Text};
use zippy_common::source::Span;

use super::format;
use crate::output::{format_code, format_note_kind};
use crate::Database;

/// Print a nicely formatted diagnostic.
pub(super) fn print_diagnostic(db: &Database, message: Message) {
    let source_name = message.span.source.name(db);
    let source = message.span.source.content(db);
    let ranges = get_line_ranges(source, message.span);

    let term = Terminal::new();

    // Print source info
    let source_name = if let Some(root) = &db.root {
        if let Ok(path) = source_name.strip_prefix(root) {
            path.display()
        } else {
            source_name.display()
        }
    } else {
        source_name.display()
    };

    eprint!("in {source_name}");
    if let Some((line, column)) = ranges.first().map(|range| (range.line, range.first_column)) {
        eprintln!(":{line}:{column}");
    } else {
        eprintln!();
    }

    // Print code and title
    let code = format_code(message.code);
    let indent = code.len() + 2;
    let title = term.indented(indent, message.title);

    eprintln!("{code}: {title}");

    // Print source lines and squigglies
    let biggest_line = ranges.iter().map(|range| range.line).max().unwrap_or(0);
    let line_number_width = count_digits(biggest_line);

    // Only print squigglies on single-line errors.
    let squigglies = ranges.len() == 1;

    for range in ranges {
        // Print line number
        eprint!(" {: >line_number_width$} | ", range.line); // arcane magic
        let indent = line_number_width + 4;

        // Print source
        let source_line = &source[range.first_byte..range.last_byte];

        let width = term.width.and_then(|width| {
            if indent > width {
                None
            } else {
                Some(width - indent)
            }
        });

        let (source_line, start, end) = format::cutoff(
            width,
            (range.first_column, range.last_column),
            source_line.to_string(),
        );
        eprintln!("{source_line}");

        // Print squiggly thingies
        if squigglies {
            eprintln!("{}{}", " ".repeat(indent + start), "^".repeat(end - start));
        }
    }

    // Print notes and helps
    let any_notes = !message.notes.is_empty();
    for (kind, note) in message.notes {
        let kind = format_note_kind(kind);
        let indent = kind.len() + 2;
        eprintln!("{kind}: {}", term.indented(indent, note));
    }

    if any_notes {
        eprintln!();
    }

    // TODO: print labels
}

pub(super) struct Terminal {
    width: Option<usize>,
}

impl Terminal {
    /// If we cannot fit more than this number of "characters" onto the terminal
    /// for whatever reason, don't attempt any indentation/justification stuff.
    pub const GIVE_UP: usize = 10;

    pub fn new() -> Self {
        let width = terminal::size().ok().map(|(width, _)| width as usize);
        Self { width }
    }

    pub fn indented(&self, indent: usize, text: Text) -> String {
        format::indented(Self::GIVE_UP, self.width, indent, text)
    }
}

/// A half-inclusive range of text within a certain line. Lines and columns are
/// both zero-indexed. Also contains the first and last byte of the line.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct LineRange {
    line: usize,
    first_column: usize,
    last_column: usize,
    first_byte: usize,
    last_byte: usize,
}

/// Count the number of (decimal) digits in the given number.
fn count_digits(mut num: usize) -> usize {
    if num == 0 {
        1
    } else {
        let mut count = 0;
        while num != 0 {
            count += 1;
            num /= 10;
        }
        count
    }
}

/// Compute the line ranges for the given span in the given text.
fn get_line_ranges(text: &str, span: Span) -> Vec<LineRange> {
    let (mut line, first_index) = find_first_line(text, span);
    let mut ranges = Vec::new();

    let mut first_byte = first_index;
    let mut last_byte = first_byte;

    let mut first_column = span.start.saturating_sub(first_index);
    let mut last_column = 0;
    let mut last_line = false;
    let mut unfinished = true;

    for (i, c) in text[first_index..].char_indices() {
        if c == '\n' {
            ranges.push(LineRange {
                line,
                first_column,
                last_column,
                first_byte,
                last_byte,
            });

            if last_line {
                unfinished = false;
                break;
            }

            line += 1;
            first_column = 0;
            last_column = 0;

            first_byte = first_index + i + 1;
            last_byte = first_index + i;
        } else {
            last_column += c.len_utf8();
        }

        if first_index + i >= span.end {
            last_line = true;
        }

        last_byte += c.len_utf8();
    }

    if unfinished {
        ranges.push(LineRange {
            line,
            first_column,
            last_column,
            first_byte,
            last_byte,
        });
    }

    ranges
}

/// Get the line and byte index of the first line that intersects with the span.
/// The line is zero-indexed, so the first line is `0`.
///
/// # Panics
///
/// Panics if the span is outside the text.
fn find_first_line(text: &str, span: Span) -> (usize, usize) {
    let mut line = 0;
    let mut line_start = 0;
    let mut inside = false;

    for (i, c) in text.char_indices() {
        if c == '\n' {
            line += 1;
            line_start = i + 1; // newlines are one byte long, this is okay
        }

        if i >= span.start {
            inside = true;
            break;
        }
    }

    assert!(inside);

    (line, line_start)
}

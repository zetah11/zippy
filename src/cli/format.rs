//! Contains various tools for formatting text within constrained widths.
//! [`indented`] reproduces the text in its entirety, adding line breaks and
//! indents where appropriate. [`cutoff`] takes a simple string and renders a
//! highlighted part, cutting off either end to make it fit within a width.
//! [`plain`] does a straightforward stringification of some [`Text`].
//!
//! These functions all operate under the assumption that every `char` has the
//! same width (like with a monospace font), which can easily break with things
//! like emojis, graphemes spanning multiple codepoints, and so on. Taking that
//! into consideration would make this significantly more complicated and could
//! only really reliably be done with knowledge of things like the font used.
//! Since this is meant to be a command-line application interacting with text
//! that, for the most part, fits these assumptions, this "best effort" is
//! considered good enough.

use itertools::Itertools;
use zippy_common::messages::{Text, TextPart};

/// Format the given string such that the span `highlight` is visible, cutting
/// off either with ellipses (`...`) if the entire line cannot fit.
///
/// ```text
/// abcdefghijkl...
/// ...ijklmnopq...
/// ...opqrstuvwxyz
/// ```
///
/// Returns the formatted string and the position of the highlight span.
pub fn cutoff(
    width: Option<usize>,
    highlight: (usize, usize),
    text: String,
) -> (String, usize, usize) {
    // Tricky conditions and their solutions:
    // - No width
    // - Width is too small to fit ellipses
    // - Width is too small to fit two ellipses
    //  => "Give up" and return the string and highlight unchanged
    // - Width is too small to fit all of the highlighted span
    //  => Keep the highlight start (possibly with ellipses), set highlight end
    //     to the ending ellipses

    let (start, end) = highlight;
    assert!(start <= end);
    assert!(end <= text.len());

    let Some(width) = width else {
        return (text, start, end);
    };

    if width <= 6 {
        return (text, start, end);
    }

    // All of the text fits on the line, no cutoff necessary
    if width >= text.len() {
        return (text, start, end);
    }

    // "Ideally", we want the highlighted span to be approximately centered
    // on the resulting line.
    let span_width = (end - start).min(width);
    let total_sides = width - span_width;

    let left_width = (total_sides + 1) / 2;

    let mut left_index = start.saturating_sub(left_width);
    let mut right_index = (left_index + width).min(text.len());

    let mut resulting_start = start - left_index;
    let resulting_end = resulting_start + span_width;

    let (mut ellipses_start, mut ellipses_end) = (false, false);
    if left_index != 0 {
        left_index += 3;
        ellipses_start = true;
    }
    if right_index != text.len() {
        right_index -= 3;
        ellipses_end = true;
    }

    // Ensure the beginning of the highlighted span is always visible.
    // This can only happen if ellipses were added to the start, so we can set
    // the resulting start to 3.
    if start < left_index {
        right_index -= left_index - start;
        left_index = start;
        resulting_start = 3;
    }

    // awaiting stabilization ................
    //let left_index = text.ceil_char_boundary(left_index);
    //let right_index = text.floor_char_boundary(right_index);

    let mut result = String::with_capacity(width);
    if ellipses_start {
        result.push_str("...");
    }
    result.push_str(&text[left_index..right_index]);
    if ellipses_end {
        result.push_str("...");
    }

    (result, resulting_start, resulting_end)
}

/// Attempt to produce a string with this `text` not taking up more than `width`
/// characters and every newline followed by `indent` spaces. The first line in
/// the result will not have any spaces prepended.
///
/// If the width is not specified, or the width without indent characters is
/// less than `give_up`, then the text will be formatted the same as `plain()`.
///
/// TODO: This should treat `Code` parts as "prefer not to break", putting them
/// on a separate line if possible, breaking at word boundaries if not, etc.
pub fn indented(give_up: usize, width: Option<usize>, indent: usize, text: Text) -> String {
    // Format the text using `plain` first so that "joins" between distinct text
    // parts are handled correctly. For instance, "`abc`d" (a code part followed
    // by a text part) should be treated as one word without a space inbetween,
    // not as two.
    let text = plain(text);

    let Some(width) = width else {
        return text;
    };

    if indent > width || width - indent < give_up {
        return text;
    }

    let indent_string = " ".repeat(indent);

    let to_fill = width - indent;
    let mut remaining = to_fill;
    let mut result = String::new();

    let mut first_word = true;

    for word in text.split(' ') {
        if word.is_empty() {
            continue;
        }

        // Only prepend a space if this is not the first word.
        let word_with_space_size = word.len() + usize::from(!first_word);

        // Is there room for the entire word on a separate line, but not in
        // the remaining space? If so, new line.
        if word_with_space_size > remaining && word.len() <= to_fill {
            result.push('\n');
            result.push_str(&indent_string);
            remaining = to_fill;
        }
        // Otherwise, no new line, so prepend a space.
        else if !first_word {
            result.push(' ');
            remaining -= 1;
        }

        // Push the characters onto the word, adding newlines and indents
        // if necessary.
        for c in word.chars() {
            if remaining == 0 {
                result.push('\n');
                result.push_str(&indent_string);
                remaining = to_fill;
            }

            result.push(c);
            remaining -= 1;
        }

        first_word = false;
    }

    result
}

/// Format the given `text` "plainly" as a continuous run of characters.
pub fn plain(text: Text) -> String {
    text.0
        .into_iter()
        .map(|part| match part {
            TextPart::Text(text) => text,
            TextPart::Code(code) => format!("`{code}`"),
        })
        .join("")
}

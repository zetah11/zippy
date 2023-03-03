use itertools::Itertools;
use zippy_common::messages::{Text, TextPart};

use crate::pretty::Prettier;

/// Format a [`Text`] into a string suitable for a language client.
pub fn format_text(prettier: &Prettier, text: Text) -> String {
    text.0
        .into_iter()
        .map(|part| format_part(prettier, part))
        .join("")
}

fn format_part(prettier: &Prettier, part: TextPart) -> String {
    match part {
        TextPart::Text(text) => text,
        TextPart::Code(code) => format!("`{code}`"),
        TextPart::Name(name) => format!("`{}`", prettier.pretty_name(name)),
    }
}

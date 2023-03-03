use itertools::Itertools;
use zippy_common::messages::{Text, TextPart};

/// Format a [`Text`] into a string suitable for a language client.
pub fn format_text(text: Text) -> String {
    text.0.into_iter().map(format_part).join("")
}

fn format_part(part: TextPart) -> String {
    match part {
        TextPart::Text(text) => text,
        TextPart::Code(code) => format!("`{code}`"),
    }
}

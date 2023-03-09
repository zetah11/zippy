//! Support for semantic tokens.

mod legend;

pub use self::legend::Legend;

use lsp_types::{SemanticToken, SemanticTokens};
use zippy_common::source::{Source, Span};
use zippy_frontend::parser::{get_tokens, Token};

use super::Backend;

impl Backend {
    pub(super) fn tokenize(&self, source: Source) -> SemanticTokens {
        let tokens = get_tokens(&self.database, source);

        let mut builder =
            SemanticTokensBuilder::new(&self.token_legend, source.content(&self.database));

        for token in tokens {
            builder.add_token(token);
        }

        builder.build()
    }
}

/// Algorithm: in order to avoid having to traverse the entire source for every
/// token, we store the result of the previous `start_byte -> line, column`
/// mapping, and only retraverse if the given span is less than the start byte.
/// Otherwise, we just traverse the string from the start_byte and compute a
/// delta.
struct SemanticTokensBuilder<'s> {
    legend: &'s Legend,
    source: &'s str,
    tokens: Vec<SemanticToken>,

    /// The start index of the previous span we traversed.
    previous: Option<usize>,
}

struct RelativeSpan {
    delta_line: usize,
    delta_column: usize,
    length: usize,
}

impl<'s> SemanticTokensBuilder<'s> {
    pub fn new(legend: &'s Legend, source: &'s str) -> Self {
        Self {
            legend,
            source,
            tokens: Vec::new(),
            previous: None,
        }
    }

    pub fn build(self) -> SemanticTokens {
        SemanticTokens {
            result_id: None,
            data: self.tokens,
        }
    }

    /// Add a token to this list of semantic tokens.
    pub fn add_token(&mut self, token: &Token) {
        for (_, span) in token.comments.iter() {
            let relative = self.translate_span(*span);
            self.previous = Some(span.start);

            let (ty, modifiers) = self.legend.for_comment();
            self.push_token(relative, ty, modifiers);
        }

        if let Some((ty, modifiers)) = self.legend.for_token(&token.kind) {
            let relative = self.translate_span(token.span);
            self.previous = Some(token.span.start);
            self.push_token(relative, ty, modifiers);
        }
    }

    fn push_token(&mut self, relative: RelativeSpan, ty: u32, modifiers: u32) {
        self.tokens.push(SemanticToken {
            delta_line: u32::try_from(relative.delta_line).expect("delta_line is reasonable"),
            delta_start: u32::try_from(relative.delta_column).expect("delta_column is reasonable"),
            length: u32::try_from(relative.length).expect("length is reasonable"),
            token_type: ty,
            token_modifiers_bitset: modifiers,
        });
    }

    /// Translate the given `Span` into a span relative to `self.previous`.
    /// This assumes that the given span belongs to the source in `self.source`.
    /// This does *not* replace `self.previous`.
    fn translate_span(&self, span: Span) -> RelativeSpan {
        let (source, mut index) = match self.previous {
            Some(previous) if previous <= span.start => (&self.source[previous..], previous),
            _ => (self.source, 0),
        };

        let mut delta_line = 0;
        let mut delta_column = 0;

        for c in source.chars() {
            if index >= span.start {
                break;
            }

            index += c.len_utf8();

            if c == '\n' {
                delta_line += 1;
                delta_column = 0;
            } else {
                delta_column += 1;
            }
        }

        RelativeSpan {
            delta_line,
            delta_column,
            length: span.length(),
        }
    }
}

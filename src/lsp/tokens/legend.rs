use lsp_types::{SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend};
use zippy_frontend::parser::TokenType;

pub struct Legend {
    types: Vec<SemanticTokenType>,
    modifiers: Vec<SemanticTokenModifier>,
}

impl Legend {
    const COMMENT: u32 = 0;
    const NUMBER: u32 = 1;
    const STRING: u32 = 2;
    const KEYWORD: u32 = 3;

    const NONE: u32 = 0;
    const DOCUMENTATION: u32 = 1;

    pub fn new() -> Self {
        let types = vec![
            SemanticTokenType::COMMENT,
            SemanticTokenType::NUMBER,
            SemanticTokenType::STRING,
            SemanticTokenType::KEYWORD,
        ];

        let modifiers = vec![SemanticTokenModifier::DOCUMENTATION];

        Self { types, modifiers }
    }

    /// Produce a semantic token legend from this.
    pub fn get_legend(&self) -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: self.types.clone(),
            token_modifiers: self.modifiers.clone(),
        }
    }

    /// Get the semantic token type and the modifiers for the given
    pub fn for_comment(&self) -> (u32, u32) {
        (Self::COMMENT, Self::NONE)
    }

    /// Get the semantic token type and the modifiers for the given token type,
    /// if it is to be highlighted.
    pub fn for_token(&self, token: &TokenType) -> Option<(u32, u32)> {
        match token {
            TokenType::Name(_) => None,
            TokenType::Number(_) => Some((Self::NUMBER, Self::NONE)),
            TokenType::String(_) => Some((Self::STRING, Self::NONE)),
            TokenType::Entry => Some((Self::KEYWORD, Self::NONE)),
            TokenType::Fun => Some((Self::KEYWORD, Self::NONE)),
            TokenType::Import => Some((Self::KEYWORD, Self::NONE)),
            TokenType::Let => Some((Self::KEYWORD, Self::NONE)),

            TokenType::Colon => None,
            TokenType::Equal => None,
            TokenType::Period => None,

            TokenType::LeftParen => None,
            TokenType::RightParen => None,
            TokenType::Semicolon => None,

            TokenType::Documentation(_) => Some((Self::COMMENT, Self::DOCUMENTATION)),

            TokenType::Indent | TokenType::Dedent | TokenType::Eol => None,
        }
    }
}

use crate::lex::Token;

pub trait Matcher {
    fn matches(&self, tok: &Token) -> bool;
}

impl Matcher for Token {
    fn matches(&self, tok: &Token) -> bool {
        match (self, tok) {
            (_, Token::Invalid) => true,
            (Token::Name(..), Token::Name(..)) => true,
            (Token::Number(..), Token::Number(..)) => true,
            (t, u) => t == u,
        }
    }
}

impl Matcher for &[Token] {
    fn matches(&self, tok: &Token) -> bool {
        self.iter().any(|other| other.matches(tok))
    }
}

impl<F: Fn(&Token) -> bool> Matcher for F {
    fn matches(&self, tok: &Token) -> bool {
        self(tok)
    }
}

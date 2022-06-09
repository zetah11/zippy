use lalrpop_util::ParseError;
use std::convert::Infallible;

use super::grammar::__ToTriple;
use crate::lex::Token;
use crate::source::Spanned;

type SpannedToken = (usize, Token, usize);
type Error = ParseError<usize, Token, Infallible>;

impl __ToTriple<'_> for Spanned<Token> {
    fn to_triple((tok, span): Self) -> Result<SpannedToken, Error> {
        Ok((span.start, tok, span.end))
    }
}

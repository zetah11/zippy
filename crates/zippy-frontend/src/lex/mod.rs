mod convert;
mod token;

use log::{info, trace};
use logos::Logos;

use convert::parse_dec;
use token::FreeToken;
use zippy_common::message::{File, Messages, Span};
use zippy_common::Driver;

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Fun,
    Let,
    Type,
    Upto,

    GroupOpen,
    GroupClose,
    Delimit,
    Pipe,

    MinArrow,
    EqArrow,

    Question,
    Comma,
    Star,

    Equal,
    Colon,

    Name(String),
    Number(u64),

    Invalid,
}

pub fn lex(driver: &mut impl Driver, src: impl AsRef<str>, file: File) -> Vec<(Token, Span)> {
    info!("lexing file with id {file}");
    let mut lexer = Lexer::new(src.as_ref(), file);
    lexer.lex();
    driver.report(lexer.msgs);
    trace!("done lexing {file}");
    lexer.res
}

impl Token {
    fn delimit_after(&self) -> bool {
        match self {
            Self::Fun
            | Self::Let
            | Self::Type
            | Self::Upto
            | Self::GroupOpen
            | Self::Pipe
            | Self::MinArrow
            | Self::EqArrow
            | Self::Comma
            | Self::Star
            | Self::Equal
            | Self::Colon
            | Self::Delimit => false,

            Self::GroupClose | Self::Question | Self::Name(_) | Self::Number(_) | Self::Invalid => {
                true
            }
        }
    }

    fn group_before(&self) -> bool {
        match self {
            Self::Upto
            | Self::GroupClose
            | Self::Pipe
            | Self::Delimit
            | Self::MinArrow
            | Self::EqArrow
            | Self::Comma
            | Self::Star
            | Self::Equal
            | Self::Colon => false,

            Self::Fun
            | Self::Let
            | Self::Type
            | Self::GroupOpen
            | Self::Question
            | Self::Name(_)
            | Self::Number(_)
            | Self::Invalid => true,
        }
    }
}

struct Lexer<'src> {
    lex: logos::SpannedIter<'src, FreeToken<'src>>,
    file: File,
    res: Vec<(Token, Span)>,
    msgs: Messages,

    indents: Vec<usize>,
    delimit_after: bool,
    last_newline: Option<(usize, Span)>,

    last_span: Option<Span>,
}

impl<'src> Lexer<'src> {
    fn new(src: &'src str, file: File) -> Self {
        Self {
            lex: FreeToken::lexer(src).spanned(),
            file,
            res: Vec::new(),
            msgs: Messages::new(),

            indents: Vec::new(),
            delimit_after: false,
            last_newline: None,

            last_span: None,
        }
    }

    fn lex(&mut self) {
        while self.dispatch() {}

        let span = self.last_span.unwrap();
        self.res
            .extend(std::iter::repeat_with(|| (Token::GroupClose, span)).take(self.indents.len()));
    }

    fn dispatch(&mut self) -> bool {
        if let Some((tok, span)) = self.lex.next() {
            let span = Span::new(self.file, span.start, span.end);
            let tok = match tok {
                FreeToken::Fun => Token::Fun,
                FreeToken::Let => Token::Let,
                FreeToken::Type => Token::Type,
                FreeToken::Upto => Token::Upto,
                FreeToken::LParen => Token::GroupOpen,
                FreeToken::RParen => Token::GroupClose,
                FreeToken::Pipe => Token::Pipe,
                FreeToken::MinArrow => Token::MinArrow,
                FreeToken::EqArrow => Token::EqArrow,
                FreeToken::Question => Token::Question,
                FreeToken::Comma => Token::Comma,
                FreeToken::Star => Token::Star,
                FreeToken::Equal => Token::Equal,
                FreeToken::Colon => Token::Colon,
                FreeToken::Name(name) => Token::Name(name.into()),
                FreeToken::DecNumber(num) => Token::Number(parse_dec(num)),

                FreeToken::Newline(indent) => {
                    self.last_newline = Some((indent, span));
                    return true;
                }

                FreeToken::Error => {
                    self.msgs.at(span).lex_invalid();
                    Token::Invalid
                }
            };

            self.last_span = Some(span);

            self.handle_newline(tok.group_before());
            self.delimit_after = tok.delimit_after();
            self.res.push((tok, span));

            true
        } else {
            false
        }
    }

    fn handle_newline(&mut self, group_before: bool) {
        if let Some((indent, span)) = self.last_newline {
            let top = self.indents.last().cloned().unwrap_or(0);

            if indent > top && group_before {
                self.indents.push(indent);
                self.res.push((Token::GroupOpen, span));
            } else {
                let mut skip = 0;
                for jndent in self.indents.iter().rev() {
                    if &indent >= jndent {
                        break;
                    }

                    skip += 1;
                }

                self.indents.truncate(self.indents.len() - skip);
                self.res
                    .extend(std::iter::repeat_with(|| (Token::GroupClose, span)).take(skip));

                if self.delimit_after && group_before {
                    self.res.push((Token::Delimit, span))
                }
            }
        }

        self.last_newline = None;
    }
}

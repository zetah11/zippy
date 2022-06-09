//! Parsing takes a stream of tokens and produces a parse tree, which represents
//! the structure of the code implicitly in the tree shape.

#![allow(missing_docs)]

pub mod hir;

mod ast;
mod convert;
mod error;
mod lower;
mod span;

use std::sync::Arc;

use lalrpop_util::{lalrpop_mod, ErrorRecovery};

use crate::lex::{Lexer, Token};
use crate::message::Message;
use crate::source::{SourceId, Span};

use error::to_message;
use grammar::DeclListParser;
use hir::Decls;
use lower::{lower_decls, ParsedData};

lalrpop_mod!(grammar, "/parse/grammar.rs");

/// See the [module-level documentation](crate::parse) for more.
#[salsa::query_group(ParserStorage)]
pub trait Parser: Lexer {
    /// Parse the given source.
    fn parse(&self, id: SourceId) -> (Arc<Decls<ParsedData>>, Arc<Vec<Message>>);

    /// Get the parse tree for a given source.
    fn parse_tree(&self, id: SourceId) -> Arc<Decls<ParsedData>>;

    /// Get the parse errors for a given source.
    fn parse_errs(&self, id: SourceId) -> Arc<Vec<Message>>;
}

fn parse_tree(db: &dyn Parser, id: SourceId) -> Arc<Decls<ParsedData>> {
    db.parse(id).0
}

fn parse_errs(db: &dyn Parser, id: SourceId) -> Arc<Vec<Message>> {
    db.parse(id).1
}

fn parse(db: &dyn Parser, id: SourceId) -> (Arc<Decls<ParsedData>>, Arc<Vec<Message>>) {
    let toks = db.lex(id);
    let (tree, errs) = parse_toks(toks, id);
    (Arc::new(tree), Arc::new(errs))
}

fn parse_toks(toks: Arc<Vec<(Token, Span)>>, at: SourceId) -> (Decls<ParsedData>, Vec<Message>) {
    let mut errors = Vec::new();
    let decls = DeclListParser::new()
        .parse(
            &mut errors,
            toks.iter()
                .map(|(tok, span)| (span.start, tok.clone(), span.end)),
        )
        .unwrap_or_else(|error| {
            errors.push(ErrorRecovery {
                error,
                dropped_tokens: Vec::new(),
            });
            Vec::new()
        });

    let decls = lower_decls(decls, at);
    let errors = errors.into_iter().map(|err| to_message(err, at)).collect();

    (decls, errors)
}

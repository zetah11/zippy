//! `zc` is the compiler library for z, responsible for parsing and analyzing
//! code for both zd and zs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod declare;
pub mod inputs;
pub mod lang;
pub mod lex;
pub mod message;
pub mod name;
pub mod parse;
pub mod source;

use salsa::{Database, ParallelDatabase, Snapshot};

use inputs::InputsStorage;
use lex::LexerStorage;
use parse::ParserStorage;

/// The main database for the compiler.
#[allow(missing_debug_implementations)]
#[salsa::database(InputsStorage, LexerStorage, ParserStorage)]
#[derive(Default)]
pub struct ZcDatabase {
    storage: salsa::Storage<Self>,
}

impl Database for ZcDatabase {}

impl ParallelDatabase for ZcDatabase {
    fn snapshot(&self) -> Snapshot<Self> {
        Snapshot::new(Self {
            storage: self.storage.snapshot(),
        })
    }
}

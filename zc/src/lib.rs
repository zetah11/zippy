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
pub mod resolve;
pub mod scope;
pub mod source;

use salsa::{Database, ParallelDatabase, Snapshot};

use declare::DeclStorage;
use inputs::InputsStorage;
use lex::LexerStorage;
use name::NameInternerStorage;
use parse::ParserStorage;
use resolve::ResolveStorage;

/// The main database for the compiler.
#[allow(missing_debug_implementations)]
#[salsa::database(
    DeclStorage,
    InputsStorage,
    LexerStorage,
    NameInternerStorage,
    ParserStorage,
    ResolveStorage
)]
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

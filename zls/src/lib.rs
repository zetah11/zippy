//! `zls` is a language server for the z language. In particular, this is `zls`
//! in library form.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod backend;
pub mod db;
pub mod file;

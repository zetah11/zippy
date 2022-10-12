mod convert;
mod free;

use crate::mir::{Context, Decls, Types};
use crate::resolve::names::Names;
use convert::Converter;

pub fn close(names: &mut Names, types: &mut Types, context: &mut Context, decls: Decls) -> Decls {
    let mut converter = Converter::new(names, context, types);
    converter.convert(decls)
}

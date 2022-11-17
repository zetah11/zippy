mod args;
mod console_driver;
//mod emit;
mod input;
mod target;

use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::Path;

use backend::c::emit;
use frontend::{parse, ParseResult};
use midend::elaborate;

use clap::Parser;
use codespan_reporting::files::SimpleFiles;

use console_driver::ConsoleDriver;

use args::Arguments;
use input::read_file;
use target::get_target;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Arguments::parse();
    let target = get_target(&args);
    println!("target {target}");

    let src = read_file(&args.path)?;

    let mut files = SimpleFiles::new();
    let file = files.add(args.path.to_string_lossy().into(), src.clone());

    let mut driver = ConsoleDriver::new(&args, files);

    let ParseResult {
        checked,
        mut names,
        entry,
    } = parse(&mut driver, src, file);

    let (types, context, decls) = elaborate(&mut driver, &mut names, checked, entry);

    let code = emit(&mut names, &types, &context, entry, decls);

    DirBuilder::new().recursive(true).create("artifacts")?;

    let mut file = File::create(Path::new("artifacts").join(args.path.with_extension("c")))?;
    file.write_all(code.as_bytes())?;

    Ok(())
}

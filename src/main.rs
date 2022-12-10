mod args;
mod compile;
mod console_driver;
mod input;
mod target;

use zippy_backend::c::emit;
use zippy_frontend::{parse, ParseResult};
use zippy_midend::elaborate;

use clap::Parser;
use codespan_reporting::files::SimpleFiles;

use console_driver::ConsoleDriver;

use self::args::Arguments;
use self::compile::compile;
use self::input::read_file;
use self::target::get_target;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Arguments::parse();
    let target = get_target(&args);

    let src = read_file(&args.path)?;

    let mut files = SimpleFiles::new();
    let file = files.add(args.path.to_string_lossy().into(), src.clone());

    let mut driver = ConsoleDriver::new(&args, files);

    let ParseResult {
        checked,
        mut names,
        entry,
    } = parse(&mut driver, src, file);

    let (mut types, context, decls) = elaborate(&mut driver, &mut names, checked, entry);

    let code = emit(&mut names, &mut types, &context, entry, decls);
    let _exec = compile(&args, &target, code)?;

    Ok(())
}

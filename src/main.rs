mod args;
mod compile;
mod console_driver;
mod input;
mod target;

use std::process::Command;

use zippy_backend::c::emit;
use zippy_frontend::{parse, ParseResult};
use zippy_midend::elaborate;

use anyhow::anyhow;
use clap::Parser;
use codespan_reporting::files::SimpleFiles;

use self::args::Arguments;
use self::compile::compile;
use self::console_driver::ConsoleDriver;
use self::input::read_file;
use self::target::get_target;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = Arguments::parse();
    let target = get_target(&args);
    let source = &args.options().path;

    let src = read_file(source)?;

    let mut files = SimpleFiles::new();
    let file = files.add(source.to_string_lossy().into(), src.clone());

    let mut driver = ConsoleDriver::new(&args, files);

    if args.command.check() {
        let ParseResult {
            checked,
            mut names,
            entry,
        } = parse(&mut driver, src, file);

        let (mut types, context, decls) = elaborate(&mut driver, &mut names, checked, entry);

        if args.command.build() {
            let code = emit(&mut names, &mut types, &context, entry, decls);
            let exec = compile(&args, &target, code)?;

            if args.command.run() {
                let status = Command::new(exec).args(args.slop.drain(..)).status()?;

                if !status.success() {
                    let err = if let Some(code) = status.code() {
                        anyhow!("program quit with code {code}")
                    } else {
                        anyhow!("program terminated by signal")
                    };

                    return Err(err);
                }
            }
        }
    }

    Ok(())
}

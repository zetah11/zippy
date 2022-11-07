mod args;
mod console_driver;
mod emit;
mod input;
mod target;

use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::Path;

use backend::asm::asm;
use backend::codegen::x64::{encode, pretty, CONSTRAINTS};
use frontend::{parse, ParseResult};
use midend::elaborate;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use codespan_reporting::files::SimpleFiles;

use console_driver::ConsoleDriver;

use args::Arguments;
use emit::{write_coff, write_elf};
use input::read_file;
use target::get_target;
use target_lexicon::{BinaryFormat, OperatingSystem, Triple};

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

    let program = asm(CONSTRAINTS, &types, &context, entry, decls);

    let code = match encode(&mut names, &target, entry, program) {
        Ok(code) => code,
        Err(error) => {
            let mut cmd = Arguments::command();
            cmd.error(ErrorKind::ValueValidation, error).exit()
        }
    };

    println!("{}", pretty(&names, &code));

    println!("{}", pretty_hex::pretty_hex(&code.result.inner.code_buffer));

    DirBuilder::new().recursive(true).create("artifacts")?;
    match target {
        Triple {
            binary_format: BinaryFormat::Elf,
            ..
        }
        | Triple {
            operating_system: OperatingSystem::Linux,
            ..
        } => {
            let elf = write_elf(&names, code);
            let mut main =
                File::create(Path::new("artifacts").join(args.path.with_extension("o")))?;
            main.write_all(&elf)?;
        }

        Triple {
            binary_format: BinaryFormat::Coff,
            ..
        }
        | Triple {
            operating_system: OperatingSystem::Windows,
            ..
        } => {
            let coff = write_coff(&names, code);
            let mut main =
                File::create(Path::new("artifacts").join(args.path.with_extension("lib")))?;
            main.write_all(&coff)?;
        }

        target => {
            let mut cmd = Arguments::command();
            cmd.error(
                ErrorKind::ValueValidation,
                format!("Unsupported output target '{target}'"),
            )
            .exit();
        }
    }

    Ok(())
}

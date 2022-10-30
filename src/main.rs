mod console_driver;
mod emit;
mod input;

use std::env;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::Path;

use backend::asm::asm;
use backend::codegen::x64::{self, codegen, Target, CONSTRAINTS};
use frontend::{parse, ParseResult};
use midend::elaborate;

use anyhow::anyhow;
use codespan_reporting::files::SimpleFiles;

use console_driver::ConsoleDriver;

use emit::{write_coff, write_elf};
use input::read_file;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = env::args();
    let invoke = args.next().unwrap();

    let path = match args.next() {
        Some(path) => path,
        None => return Err(anyhow!("usage: {invoke} <path>")),
    };

    let src = read_file(Path::new(&path))?;

    let mut files = SimpleFiles::new();
    let file = files.add(path.clone(), src.clone());

    let mut driver = ConsoleDriver::new(files);

    let ParseResult {
        checked,
        mut names,
        entry,
    } = parse(&mut driver, src, file);

    let (types, context, decls) = elaborate(&mut driver, &mut names, checked, entry);

    let program = asm(CONSTRAINTS, &types, &context, entry, decls);

    let target = Target::Windows64;
    let code = codegen(&mut names, target, entry, program);
    println!("{}", x64::pretty_program(&names, &code));

    let code = x64::encode(code);
    for byte in code.code.iter() {
        print!("{byte:02x} ");
    }

    println!();

    DirBuilder::new().recursive(true).create("artifacts")?;
    match target {
        Target::Linux64 => {
            let elf = write_elf(&names, code);
            let mut main =
                File::create(Path::new("artifacts").join(Path::new(&path).with_extension("o")))?;
            main.write_all(&elf)?;
        }

        Target::Windows64 => {
            let coff = write_coff(&names, code);
            let mut main =
                File::create(Path::new("artifacts").join(Path::new(&path).with_extension("lib")))?;
            main.write_all(&coff)?;
        }
    }

    Ok(())
}

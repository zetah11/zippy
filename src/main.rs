mod console_driver;

use std::collections::HashMap;
use std::env;
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::path::Path;

use codespan_reporting::files::SimpleFiles;
use corollary::asm::asm;
use corollary::codegen::x64::{self, codegen, Encoded, Target, CONSTRAINTS};
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::parse::parse;
use corollary::resolve::names::{Actual, Names};
use corollary::resolve::{resolve, ResolveRes};
use corollary::tyck::typeck;

use anyhow::anyhow;
use object::write;

use console_driver::ConsoleDriver;

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

    let toks = lex(&mut driver, src, file);
    let decls = parse(&mut driver, toks, file);
    let ResolveRes {
        decls,
        mut names,
        entry,
    } = resolve(&mut driver, decls);

    let tyckres = typeck(&mut driver, decls);
    let (types, context, decls) = elaborate(&mut driver, &mut names, tyckres, entry);

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

fn read_file(path: &Path) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn write_elf(names: &Names, code: Encoded) -> Vec<u8> {
    let mut writer = write::Object::new(
        object::BinaryFormat::Elf,
        object::Architecture::X86_64,
        object::Endianness::Little,
    );

    let text = writer.segment_name(write::StandardSegment::Text);
    let name = b".text";
    let code_section = writer.add_section(text.into(), (*name).into(), object::SectionKind::Text);

    let mut symbols = HashMap::new();

    for (symbol, (offset, size)) in code.symbols {
        let name = match &names.get_path(&symbol).1 {
            Actual::Lit(name) => name.clone(),
            Actual::Generated(gen) => gen.to_string("f"),
            Actual::Scope(_) => unreachable!(),
        };

        println!("{name} {offset} {size}");

        let id = writer.add_symbol(write::Symbol {
            name: Vec::from(name),
            value: offset as u64,
            size: size as u64,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Linkage,
            weak: false,
            section: write::SymbolSection::Section(code_section),
            flags: object::SymbolFlags::None,
        });

        symbols.insert(symbol, id);
    }

    writer.set_section_data(code_section, code.code, 8);

    for (name, relocations) in code.relocations {
        let symbol = *symbols.get(&name).unwrap();

        for relocation in relocations {
            let relocation = match relocation.kind {
                x64::RelocationKind::Absolute => write::Relocation {
                    offset: relocation.at as u64,
                    size: 64,
                    kind: object::RelocationKind::Absolute,
                    encoding: object::RelocationEncoding::Generic,
                    symbol,
                    addend: 0,
                },

                x64::RelocationKind::RelativeNext => write::Relocation {
                    offset: relocation.at as u64,
                    size: 32,
                    kind: object::RelocationKind::Relative,
                    encoding: object::RelocationEncoding::Generic,
                    symbol,
                    addend: -4, // ???
                },

                x64::RelocationKind::Relative => todo!(),
            };

            writer.add_relocation(code_section, relocation).unwrap();
        }
    }

    writer.write().unwrap()
}

fn write_coff(names: &Names, code: Encoded) -> Vec<u8> {
    let mut writer = write::Object::new(
        object::BinaryFormat::Coff,
        object::Architecture::X86_64,
        object::Endianness::Little,
    );

    let text = writer.segment_name(write::StandardSegment::Text);
    let name = b".text";
    let code_section = writer.add_section(text.into(), (*name).into(), object::SectionKind::Text);

    let mut symbols = HashMap::new();

    for (symbol, (offset, size)) in code.symbols {
        let name = match &names.get_path(&symbol).1 {
            Actual::Lit(name) => name.clone(),
            Actual::Generated(gen) => gen.to_string("_f"),
            Actual::Scope(_) => unreachable!(),
        };

        println!("{name} {offset} {size}");

        let id = writer.add_symbol(write::Symbol {
            name: Vec::from(name),
            value: offset as u64,
            size: size as u64,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Linkage,
            weak: false,
            section: write::SymbolSection::Section(code_section),
            flags: object::SymbolFlags::None,
        });

        symbols.insert(symbol, id);
    }

    writer.set_section_data(code_section, code.code, 8);

    for (name, relocations) in code.relocations {
        let symbol = match symbols.get(&name) {
            Some(symbol) => *symbol,
            None => {
                let name = match &names.get_path(&name).1 {
                    Actual::Lit(name) => name.clone(),
                    Actual::Generated(gen) => gen.to_string("_f"),
                    Actual::Scope(_) => unreachable!(),
                };

                println!("extern {name}");

                writer.add_symbol(write::Symbol {
                    name: Vec::from(name),
                    value: 0,
                    size: 0,
                    kind: object::SymbolKind::Text,
                    scope: object::SymbolScope::Linkage,
                    weak: true,
                    section: write::SymbolSection::Undefined,
                    flags: object::SymbolFlags::None,
                })
            }
        };

        for relocation in relocations {
            let relocation = match relocation.kind {
                x64::RelocationKind::Absolute => write::Relocation {
                    offset: relocation.at as u64,
                    size: 64,
                    kind: object::RelocationKind::Absolute,
                    encoding: object::RelocationEncoding::Generic,
                    symbol,
                    addend: 0,
                },

                x64::RelocationKind::RelativeNext => write::Relocation {
                    offset: relocation.at as u64,
                    size: 32,
                    kind: object::RelocationKind::Relative,
                    encoding: object::RelocationEncoding::Generic,
                    symbol,
                    addend: -4, // ???
                },

                x64::RelocationKind::Relative => todo!(),
            };

            writer.add_relocation(code_section, relocation).unwrap();
        }
    }

    writer.write().unwrap()
}

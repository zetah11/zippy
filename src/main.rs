mod console_driver;

use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::Write;

use codespan_reporting::files::SimpleFiles;
use corollary::asm::asm;
use corollary::codegen::x64::{self, codegen, Encoded, CONSTRAINTS};
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::parse::parse;
use corollary::resolve::names::{Actual, Names};
use corollary::resolve::{resolve, ResolveRes};
use corollary::tyck::typeck;

use object::write;

use console_driver::ConsoleDriver;

fn main() {
    env_logger::init();

    let src = r#"
        let main: 0 upto 1 -> ? =
          ? => apply (id, 5)
        
        let id = x => apply ((y => y), x)

        let apply: (10 -> 10) * 10 -> 10 =
          (f, x) => f x
    "#;

    let mut files = SimpleFiles::new();
    let file = files.add("main.z".into(), src.into());

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

    let code = codegen(&mut names, entry, program);
    println!("{}", x64::pretty_program(&names, &code));

    let code = x64::encode(code);
    for byte in code.code.iter() {
        print!("{byte:02x} ");
    }

    println!();

    DirBuilder::new()
        .recursive(true)
        .create("artifacts")
        .unwrap();
    let elf = write_elf(&names, code);
    let mut main = File::create("artifacts/main.o").unwrap();
    main.write_all(&elf).unwrap();
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

use std::collections::HashMap;

use backend::codegen::x64::{self, Encoded};
use backend::mangle::mangle;
use common::names::Names;

use object::write;

pub fn write_coff(names: &Names, code: Encoded) -> Vec<u8> {
    let mut writer = write::Object::new(
        object::BinaryFormat::Coff,
        object::Architecture::X86_64,
        object::Endianness::Little,
    );

    write(&mut writer, names, code);
    writer.write().unwrap()
}

pub fn write_elf(names: &Names, code: Encoded) -> Vec<u8> {
    let mut writer = write::Object::new(
        object::BinaryFormat::Elf,
        object::Architecture::X86_64,
        object::Endianness::Little,
    );

    write(&mut writer, names, code);
    writer.write().unwrap()
}

fn write(writer: &mut write::Object, names: &Names, code: Encoded) {
    let text = writer.segment_name(write::StandardSegment::Text);
    let data = writer.segment_name(write::StandardSegment::Data);
    let code_name = b".text";
    let data_name = b".rodata";
    let code_section =
        writer.add_section(text.into(), (*code_name).into(), object::SectionKind::Text);
    let data_section = writer.add_section(
        data.into(),
        (*data_name).into(),
        object::SectionKind::ReadOnlyData,
    );

    let mut symbols = HashMap::new();

    for (name, (offset, size)) in code.code_symbols {
        let mangled = mangle(names, &name);
        let symbol = SymbolBuilder::new(mangled)
            .with_value(offset, size)
            .with_code_section(code_section)
            .build();
        let symbol = writer.add_symbol(symbol);

        symbols.insert(name, symbol);
    }

    for (name, (offset, size)) in code.data_symbols {
        let mangled = mangle(names, &name);
        let symbol = SymbolBuilder::new(mangled)
            .with_value(offset, size)
            .with_kind(object::SymbolKind::Data)
            .with_code_section(data_section)
            .build();
        let symbol = writer.add_symbol(symbol);

        symbols.insert(name, symbol);
    }

    writer.set_section_data(code_section, code.code, 8);
    writer.set_section_data(data_section, code.data, 8);

    for (name, relocations) in code.relocations {
        let symbol = match symbols.get(&name) {
            Some(symbol) => *symbol,
            None => {
                let mangled = mangle(names, &name);

                let symbol = SymbolBuilder::new(mangled).build();
                let symbol = writer.add_symbol(symbol);

                symbols.insert(name, symbol);
                symbol
            }
        };

        for relocation in relocations {
            let relocation = make_relocation(symbol, relocation);
            writer.add_relocation(code_section, relocation).unwrap();
        }
    }
}

fn make_relocation(symbol: write::SymbolId, relocation: x64::Relocation) -> write::Relocation {
    match relocation.kind {
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
    }
}

#[derive(Debug)]
struct SymbolBuilder {
    name: String,
    value: u64,
    size: u64,
    kind: object::SymbolKind,
    scope: object::SymbolScope,
    section: write::SymbolSection,
}

impl SymbolBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: 0,
            size: 0,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Linkage,
            section: write::SymbolSection::Undefined,
        }
    }

    pub fn with_code_section(mut self, section: write::SectionId) -> Self {
        self.section = write::SymbolSection::Section(section);
        self
    }

    pub fn with_kind(mut self, kind: object::SymbolKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_value(mut self, value: usize, size: usize) -> Self {
        self.value = value as u64;
        self.size = size as u64;
        self
    }

    pub fn build(self) -> write::Symbol {
        write::Symbol {
            name: Vec::from(self.name),
            value: self.value,
            size: self.size,
            kind: self.kind,
            scope: self.scope,
            weak: false,
            section: self.section,
            flags: object::SymbolFlags::None,
        }
    }
}

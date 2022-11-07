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

fn write(writer: &mut write::Object, names: &Names, mut code: Encoded) {
    let text = writer.segment_name(write::StandardSegment::Text);
    //let data = writer.segment_name(write::StandardSegment::Data);
    let code_name = b".text";
    //let data_name = b".rodata";
    let code_section =
        writer.add_section(text.into(), (*code_name).into(), object::SectionKind::Text);

    /*
    let data_section = writer.add_section(
        data.into(),
        (*data_name).into(),
        object::SectionKind::ReadOnlyData,
    );
    */

    let mut symbols = HashMap::new();

    for (name, label) in code.labels {
        let mangled = mangle(names, &name);
        let address = code.result.label_ip(&label).unwrap();
        let symbol = SymbolBuilder::new(mangled)
            .with_value(address, 0)
            .with_section(code_section)
            .build();
        let symbol = writer.add_symbol(symbol);

        symbols.insert(name, symbol);
    }

    writer.set_section_data(
        code_section,
        code.result.inner.code_buffer.drain(..).collect::<Vec<_>>(),
        8,
    );
    //writer.set_section_data(data_section, code.data, 8);

    for (name, (_, relocations)) in code.relocs {
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

        for (relocation, label) in relocations {
            // terrible, horrible, awful
            let address = code.result.label_ip(&label).unwrap() + 1;

            let relocation = make_relocation(symbol, address, relocation);
            writer.add_relocation(code_section, relocation).unwrap();
        }
    }
}

fn make_relocation(
    symbol: write::SymbolId,
    address: u64,
    relocation: x64::RelocationKind,
) -> write::Relocation {
    match relocation {
        x64::RelocationKind::RelativeNext => write::Relocation {
            offset: address,
            size: 32,
            kind: object::RelocationKind::Relative,
            encoding: object::RelocationEncoding::Generic,
            symbol,
            // WHAT?
            // I need to figure this out; somehow, if I use an addend of `0`, the linker
            // reports an addend of `4`, `-1` becomes `3`, `-2` becomes `2` and `-3` becomes
            // `1`. So you might, in your normal, sensible understanding of the world think
            // that `-4` would produce `0` but actually it produces `0xfffffff3`. Of course.
            // So this number right here, this reasonable `0xfffffffc`, was found through
            // brute force. I do not understand why this works, why `-4` doesn't (or even
            // why tf `0` is insufficient to begin with. I thought this was relative to the
            // next instruction???)
            addend: 0xfffffffc, // ??? ?????????? ???
        },
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

    pub fn with_section(mut self, section: write::SectionId) -> Self {
        self.section = write::SymbolSection::Section(section);
        self
    }

    /*
    pub fn with_kind(mut self, kind: object::SymbolKind) -> Self {
        self.kind = kind;
        self
    }
    */

    pub fn with_value(mut self, value: u64, size: u64) -> Self {
        self.value = value;
        self.size = size;
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

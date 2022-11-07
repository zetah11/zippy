use std::collections::HashMap;

use common::names::Names;
use iced_x86::{
    Decoder, DecoderOptions, Formatter, NasmFormatter, NumberBase, SymbolResolver, SymbolResult,
};

use super::Encoded;
use crate::mangle::mangle;

pub fn pretty(names: &Names, code: &Encoded) -> String {
    let resolver = Resolver::new(names, code);
    let mut formatter = NasmFormatter::with_options(Some(Box::new(resolver.clone())), None);
    formatter.options_mut().set_number_base(NumberBase::Decimal);

    let decoded = Decoder::new(64, &code.result.inner.code_buffer[..], DecoderOptions::NONE);

    let mut result = String::new();

    for instruction in decoded {
        let ip = instruction.ip();
        if let Some(name) = resolver.get(ip) {
            result.push_str(name);
            result.push_str(":\n");
        }

        result.push_str("    ");
        formatter.format(&instruction, &mut result);
        result.push('\n');
    }

    result
}

#[derive(Clone, Debug)]
struct Resolver {
    map: HashMap<u64, String>,
}

impl Resolver {
    pub fn new(names: &Names, code: &Encoded) -> Self {
        let mut map = HashMap::with_capacity(code.labels.len());

        for (name, label) in code.labels.iter() {
            assert!(code.info.is_intern(name));

            let name = mangle(names, name);
            let address = code.result.label_ip(label).unwrap();

            map.insert(address, name);
        }

        for (name, (address, _)) in code.relocs.iter() {
            assert!(code.info.is_extern(name));

            let name = mangle(names, name);

            map.insert(*address, name);
        }

        Self { map }
    }

    pub fn get(&self, address: u64) -> Option<&str> {
        self.map.get(&address).map(|name| name.as_str())
    }
}

impl SymbolResolver for Resolver {
    fn symbol(
        &mut self,
        _: &iced_x86::Instruction,
        _: u32,
        _: Option<u32>,
        address: u64,
        _: u32,
    ) -> Option<iced_x86::SymbolResult<'_>> {
        self.map
            .get(&address)
            .map(|name| SymbolResult::with_str(address, name))
    }
}

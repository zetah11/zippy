mod block;
mod statement;
mod types;
mod value;

use std::collections::HashMap;

use common::mir::{Context, Decls, Type, TypeId, Types};
use common::names::{Name, Names};

use crate::mangle::mangle;

pub fn emit(
    names: &mut Names,
    types: &Types,
    context: &Context,
    entry: Option<Name>,
    decls: Decls,
) -> String {
    let mut emitter = Emitter::new(names, types, context);
    emitter.emit_decls(decls);

    if let Some(entry) = entry {
        emitter.emit_entry(entry);
    }

    emitter.build()
}

#[derive(Debug)]
struct Emitter<'a> {
    res: String,
    typedefs: String,
    type_map: HashMap<TypeId, String>,
    type_name: usize,

    names: &'a mut Names,
    types: &'a Types,
    context: &'a Context,
}

impl<'a> Emitter<'a> {
    pub fn new(names: &'a mut Names, types: &'a Types, context: &'a Context) -> Self {
        Self {
            res: String::new(),
            typedefs: String::new(),
            type_map: HashMap::new(),
            type_name: 0,

            names,
            types,
            context,
        }
    }

    pub fn build(mut self) -> String {
        self.typedefs.push_str(&self.res);
        self.typedefs
    }

    pub fn emit_decls(&mut self, decls: Decls) {
        assert!(decls.defs.is_empty());

        self.res.push_str("// declarations\n");

        for (name, _) in decls.values.iter() {
            self.define_value(name);
            self.declaration();
        }

        for (name, (params, _)) in decls.functions.iter() {
            self.define_function(name, params);
            self.declaration();
        }

        self.res.push_str("\n// definitions\n");

        for (name, value) in decls.values.into_iter() {
            let value = self.emit_value(value);
            self.define_value(&name);
            self.res.push_str(&format!(" = {value};\n"));
        }

        for (name, (params, block)) in decls.functions.into_iter() {
            self.define_function(&name, &params);
            let lines = self.emit_block(name, block).join("\n\t");

            self.res.push_str(&format!(" {{\n\t{lines}\n}}\n"));
        }
    }

    pub fn emit_entry(&mut self, entry: Name) {
        let entry = mangle(self.names, &entry);

        self.res.push_str("int main(void) {\n");
        self.res.push_str(&format!("\treturn (int){entry}(0);\n"));
        self.res.push_str("}\n");
    }

    fn define_value(&mut self, name: &Name) {
        let mangled = mangle(self.names, name);

        let ty = self.context.get(name);
        let ty = self.typename(&ty).to_string();

        self.define(&mangled, &ty, "");
    }

    fn define_function(&mut self, name: &Name, params: &[Name]) {
        let mangled = mangle(self.names, name);

        let ty = self.context.get(name);
        let Type::Fun(args, rets) = self.types.get(&ty) else { unreachable!(); };

        let ret = match &rets[..] {
            [] => "void".to_string(),
            [ret] => self.typename(ret).to_string(),
            _ => self.make_struct(rets),
        };

        let args: Vec<_> = args
            .iter()
            .zip(params.iter())
            .map(|(ty, name)| {
                let name = mangle(self.names, name);
                let ty = self.typename(ty);
                format!("{ty} {name}")
            })
            .collect();

        let args = if args.is_empty() {
            "void".to_string()
        } else {
            format!("({})", args.join(", "))
        };

        self.define(&mangled, &ret, &args);
    }

    fn declaration(&mut self) {
        self.res.push_str(";\n");
    }

    fn define(&mut self, name: &str, pre: &str, post: &str) {
        self.res.push_str(&format!("static {pre} {name}{post}"));
    }
}

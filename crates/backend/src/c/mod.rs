mod block;
mod statement;
mod types;
mod value;

use std::collections::{HashMap, HashSet};

use common::mir::{discover, Context, Decls, Type, TypeId, Types};
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
    emitter.emit_decls(entry, decls);

    if let Some(entry) = entry {
        emitter.emit_entry(entry);
    }

    emitter.build()
}

#[derive(Debug)]
struct Emitter<'a> {
    res: String,
    inits: String,
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
            inits: String::new(),
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

    pub fn emit_decls(&mut self, entry: Option<Name>, decls: Decls) {
        assert!(decls.defs.is_empty());

        let reachable: HashSet<_> = if entry.is_some() {
            discover(entry, &decls).into_iter().collect()
        } else {
            self.res.push_str("// (no code generated)\n");
            return;
        };

        self.res.push_str("// declarations\n");

        for (name, value) in decls.values.iter() {
            if !reachable.contains(name) {
                continue;
            }

            self.define_value(value.needs_late_init(), name);
            self.declaration();
        }

        for (name, (params, _)) in decls.functions.iter() {
            if !reachable.contains(name) {
                continue;
            }

            self.define_function(name, params);
            self.declaration();
        }

        self.res.push_str("\n// definitions\n");

        for (name, value) in decls.values.into_iter() {
            if !reachable.contains(&name) {
                continue;
            }

            let mutable = value.needs_late_init();
            if let Some(value) = self.emit_static_value(name, value) {
                self.define_value(mutable, &name);
                self.res.push_str(&format!(" = {value};\n"));
            }
        }

        for (name, (params, block)) in decls.functions.into_iter() {
            if !reachable.contains(&name) {
                continue;
            }

            self.define_function(&name, &params);
            let lines = self.emit_block(name, None, block).join("\n\t");

            self.res.push_str(&format!(" {{\n\t{lines}\n}}\n"));
        }
    }

    pub fn emit_entry(&mut self, entry: Name) {
        let entry = mangle(self.names, &entry);

        let needs_init = !self.inits.is_empty();

        if needs_init {
            self.res.push_str("void init(void) {\n");
            self.res.push_str(&self.inits);
            self.res.push_str("}\n");
        }

        self.res.push_str("int main(void) {\n");
        if needs_init {
            self.res.push_str("\tinit();\n");
        }
        self.res.push_str(&format!("\treturn (int){entry}(0);\n"));
        self.res.push_str("}\n");
    }

    fn define_value(&mut self, mutable: bool, name: &Name) {
        let mangled = mangle(self.names, name);

        let ty = self.context.get(name);
        let ty = self.typename(&ty);
        let pre = if mutable {
            ty.to_string()
        } else {
            format!("const {ty}")
        };

        self.define(&mangled, &pre, "");
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

mod block;
mod statement;
mod types;
mod value;

use std::collections::{HashMap, HashSet};

use zippy_common::message::Messages;
use zippy_common::mir::{discover, Context, Decls, StaticValue, Type, TypeId, Types};
use zippy_common::names::{Name, Names};
use zippy_common::Driver;

use crate::mangle::mangle;

pub fn emit(
    driver: &mut impl Driver,
    names: &mut Names,
    types: &mut Types,
    context: &Context,
    entry: Option<Name>,
    decls: Decls,
) -> String {
    let mut emitter = Emitter::new(names, types, context);
    emitter.emit_decls(entry, decls);

    if let Some(entry) = entry {
        emitter.emit_entry(entry);
    }

    driver.report(emitter.messages.drain());

    emitter.build()
}

#[derive(Debug)]
struct Emitter<'a> {
    includes: HashSet<&'static str>,
    inits: String,
    auxilliary: String,
    res: String,
    typedefs: String,

    messages: Messages,

    type_map: HashMap<TypeId, String>,
    type_name: usize,
    values: HashMap<Name, StaticValue>,

    has_invalid: bool,

    names: &'a mut Names,
    types: &'a mut Types,
    context: &'a Context,
}

impl<'a> Emitter<'a> {
    pub fn new(names: &'a mut Names, types: &'a mut Types, context: &'a Context) -> Self {
        Self {
            includes: HashSet::new(),
            inits: String::new(),
            auxilliary: String::new(),
            res: String::new(),
            typedefs: String::new(),

            messages: Messages::new(),

            type_map: HashMap::new(),
            type_name: 0,
            values: HashMap::new(),

            has_invalid: false,

            names,
            types,
            context,
        }
    }

    pub fn build(self) -> String {
        let mut result = String::new();
        for include in self.includes {
            result.push_str(&format!("#include <{include}>\n"));
        }

        result.push_str(&self.auxilliary);
        result.push_str(&self.typedefs);
        result.push_str(&self.res);

        result
    }

    pub fn emit_decls(&mut self, entry: Option<Name>, decls: Decls) {
        assert!(decls.defs.is_empty());

        let (reachable, in_types) = if entry.is_some() {
            discover(self.types, self.context, entry, &decls)
        } else {
            self.res.push_str("// (no code generated)\n");
            return;
        };

        let reachable: HashSet<_> = reachable.into_iter().collect();

        self.res.push_str("// declarations\n");

        for name in in_types {
            let Some(value) = decls.values.get(&name) else { unreachable!(); };
            self.values.insert(name, value.clone());
        }

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
        let Type::Fun(args, mut rets) = self.types.get(&ty).clone() else { unreachable!(); };

        let ret = match &rets[..] {
            [] => "void".to_string(),

            [_] => {
                let ret = rets.remove(0);
                self.typename(&ret).to_string()
            }

            _ => {
                let ret = self.types.add(Type::Product(rets));
                self.typename(&ret).to_string()
            }
        };

        let args: Vec<_> = args
            .into_iter()
            .zip(params.iter())
            .map(|(ty, name)| {
                let name = mangle(self.names, name);
                let ty = self.typename(&ty);
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

    fn invalid(&mut self) -> &'static str {
        let name = "invalid";

        if !self.has_invalid {
            self.has_invalid = true;

            self.includes.insert("signal.h");
            self.auxilliary.push_str("void * invalid(void) {\n");
            self.auxilliary.push_str("\traise(SIGABRT);\n");
            self.auxilliary.push_str("\treturn (void *) 0;\n");
            self.auxilliary.push_str("}\n");
        }

        name
    }
}

use common::mir::{Statement, StmtNode};
use common::names::Name;

use crate::mangle::mangle;

use super::Emitter;

impl Emitter<'_> {
    pub fn emit_stmt(&mut self, ctx: Name, stmt: Statement) -> Vec<String> {
        let mut res = Vec::new();

        match stmt.node {
            StmtNode::Apply { names, fun, args } => {
                let fun = mangle(self.names, &fun);
                let args: Vec<_> = args
                    .into_iter()
                    .map(|value| self.emit_value(value))
                    .collect();
                let args = args.join(", ");

                let call = format!("{fun}({args});");

                match &names[..] {
                    [] => res.push(call),

                    [name] => {
                        let mangled = mangle(self.names, name);

                        let ty = self.context.get(name);
                        let ty = self.typename(&ty);

                        res.push(format!("{ty} {mangled} = {call}"));
                    }

                    _ => {
                        let target = self.names.fresh(stmt.span, ctx);
                        let target = mangle(self.names, &target);
                        let ty = self.typename(&stmt.ty);

                        res.push(format!("{ty} {target} = {call}"));

                        for (index, name) in names.into_iter().enumerate() {
                            let mangled = mangle(self.names, &name);
                            let ty = self.context.get(&name);
                            let ty = self.typename(&ty);

                            res.push(format!("{ty} {mangled} = {target}.f{index};"));
                        }
                    }
                }
            }

            StmtNode::Tuple { name, values } => {
                let values: Vec<_> = values
                    .into_iter()
                    .map(|value| self.emit_value(value))
                    .collect();
                let values = values.join(", ");

                let mangled = mangle(self.names, &name);

                let ty = self.context.get(&name);
                let ty = self.typename(&ty);

                res.push(format!("{ty} {mangled} = {{ {values} }};"));
            }

            StmtNode::Proj { name, of, at } => {
                let of = mangle(self.names, &of);
                let mangled = mangle(self.names, &name);

                let ty = self.context.get(&name);
                let ty = self.typename(&ty);

                res.push(format!("{ty} {mangled} = {of}.f{at};"));
            }

            StmtNode::Join { .. } => todo!(),

            // Hoisting should remove these
            StmtNode::Function { .. } => unreachable!(),
        }

        res
    }
}

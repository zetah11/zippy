use common::mir::{Block, BranchNode};
use common::names::Name;

use crate::mangle::mangle;

use super::Emitter;

impl Emitter<'_> {
    /// Emit the lines of a block. Should be indented.
    pub fn emit_block(&mut self, ctx: Name, block: Block) -> Vec<String> {
        let mut res = Vec::new();

        for stmt in block.exprs {
            res.extend(self.emit_stmt(ctx, stmt));
        }

        match block.branch.node {
            BranchNode::Return(rets) => match &rets[..] {
                [] => res.push("return;".into()),
                [value] => {
                    let value = self.emit_value(value.clone()); // oof
                    res.push(format!("return {value};"));
                }

                _ => {
                    let var = self.names.fresh(block.branch.span, ctx);
                    let var = mangle(self.names, &var);

                    let rets: Vec<_> = rets
                        .into_iter()
                        .map(|value| self.emit_value(value))
                        .collect();
                    let rets = rets.join(", ");

                    let ty = self.typename(&block.branch.ty);

                    res.push(format!("{ty} {var} = {{ {rets} }};"));
                    res.push(format!("return {var};"));
                }
            },

            BranchNode::Jump(..) => todo!(),
        }

        res
    }
}

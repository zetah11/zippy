use std::collections::HashMap;

use log::trace;

use super::{Block, BranchNode, Decls, Statement, StmtNode, Value, ValueNode};
use crate::names::Name;

/// Get a list of all of the names reachable from the entry point.
pub fn discover(entry: Option<Name>, decls: &Decls) -> Vec<Name> {
    let mut discoverer = MirDiscoverer::new(entry);
    discoverer.discover_decls(decls);
    discoverer.names
}

#[derive(Debug)]
struct MirDiscoverer {
    worklist: Vec<Name>,
    names: Vec<Name>,
}

impl MirDiscoverer {
    pub fn new(entry: Option<Name>) -> Self {
        Self {
            worklist: match entry {
                Some(entry) => vec![entry],
                None => Vec::new(),
            },
            names: Vec::new(),
        }
    }

    pub fn discover_decls(&mut self, decls: &Decls) {
        let defs: HashMap<_, _> = decls.defs.iter().map(|def| (def.name, def)).collect();

        while let Some(name) = self.worklist.pop() {
            if self.names.contains(&name) {
                continue;
            }

            self.names.push(name);
            if let Some(&def) = defs.get(&name) {
                self.discover_exprs(&def.bind);
            } else if let Some(value) = decls.values.get(&name) {
                self.discover_value(value);
            } else if let Some((_, body)) = decls.functions.get(&name) {
                self.discover_exprs(body);
            }
        }

        trace!("discovered {} names reachable from entry", self.names.len());
    }

    fn discover_exprs(&mut self, exprs: &Block) {
        for expr in exprs.exprs.iter() {
            self.discover_expr(expr);
        }

        match &exprs.branch.node {
            BranchNode::Return(values) => {
                values.iter().for_each(|value| self.discover_value(value));
            }
            BranchNode::Jump(target, value) => {
                self.worklist.push(*target);
                self.discover_value(value);
            }
        }
    }

    fn discover_expr(&mut self, expr: &Statement) {
        match &expr.node {
            StmtNode::Function { body, .. } => {
                self.discover_exprs(body);
            }

            StmtNode::Apply { fun, args, .. } => {
                self.worklist.push(*fun);
                args.iter().for_each(|arg| self.discover_value(arg));
            }

            StmtNode::Join { .. } => todo!(),

            StmtNode::Tuple { values, .. } => {
                values.iter().for_each(|value| self.discover_value(value));
            }

            StmtNode::Proj { of, .. } => {
                self.worklist.push(*of);
            }
        }
    }

    fn discover_value(&mut self, value: &Value) {
        match &value.node {
            ValueNode::Int(_) | ValueNode::Invalid => {}
            ValueNode::Name(name) => self.worklist.push(*name),
        }
    }
}

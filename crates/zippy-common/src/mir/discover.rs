use std::collections::{HashMap, HashSet};

use log::trace;

use super::tree::{StaticValue, StaticValueNode};
use super::{
    Block, BranchNode, Context, Decls, Statement, StmtNode, Type, TypeId, Types, Value, ValueNode,
};
use crate::names::Name;

/// Get a list of all of the names reachable from the entry point, as well as
/// all of the names which are directly used by types.
pub fn discover(
    types: &Types,
    context: &Context,
    entry: Option<Name>,
    decls: &Decls,
) -> (Vec<Name>, HashSet<Name>) {
    let mut discoverer = MirDiscoverer::new(types, context, entry);
    discoverer.discover_decls(decls);
    (discoverer.names, discoverer.in_types)
}

#[derive(Debug)]
struct MirDiscoverer<'a> {
    types: &'a Types,
    context: &'a Context,

    worklist: Vec<Name>,
    names: Vec<Name>,

    in_types: HashSet<Name>,
    visited_types: HashSet<TypeId>,
}

impl<'a> MirDiscoverer<'a> {
    pub fn new(types: &'a Types, context: &'a Context, entry: Option<Name>) -> Self {
        Self {
            types,
            context,

            worklist: match entry {
                Some(entry) => vec![entry],
                None => Vec::new(),
            },
            names: Vec::new(),

            in_types: HashSet::new(),
            visited_types: HashSet::new(),
        }
    }

    pub fn discover_decls(&mut self, decls: &Decls) {
        let defs: HashMap<_, _> = decls.defs.iter().map(|def| (def.name, def)).collect();

        while let Some(name) = self.worklist.pop() {
            if self.names.contains(&name) {
                continue;
            }

            let ty = self.context.get(&name);
            self.discover_type(&ty);

            self.names.push(name);
            if let Some(&def) = defs.get(&name) {
                self.discover_block(&def.bind);
            } else if let Some(value) = decls.values.get(&name) {
                self.discover_static_value(value);
            } else if let Some((_, body)) = decls.functions.get(&name) {
                self.discover_block(body);
            }
        }

        trace!("discovered {} names reachable from entry", self.names.len());
    }

    fn discover_block(&mut self, exprs: &Block) {
        self.discover_type(&exprs.ty);

        for expr in exprs.stmts.iter() {
            self.discover_stmt(expr);
        }

        self.discover_type(&exprs.branch.ty);

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

    fn discover_stmt(&mut self, expr: &Statement) {
        self.discover_type(&expr.ty);

        match &expr.node {
            StmtNode::Function { body, .. } => {
                self.discover_block(body);
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

            StmtNode::Coerce { of, from, .. } => {
                self.worklist.push(*of);
                self.discover_type(from);
            }
        }
    }

    fn discover_static_value(&mut self, value: &StaticValue) {
        self.discover_type(&value.ty);

        match &value.node {
            StaticValueNode::Num(_) => {}
            StaticValueNode::LateInit(block) => self.discover_block(block),
        }
    }

    fn discover_value(&mut self, value: &Value) {
        self.discover_type(&value.ty);

        match &value.node {
            ValueNode::Num(_) | ValueNode::Invalid => {}
            ValueNode::Name(name) => self.worklist.push(*name),
        }
    }

    fn discover_type(&mut self, ty: &TypeId) {
        if !self.visited_types.insert(*ty) {
            return;
        }

        match self.types.get(ty) {
            Type::Fun(ts, us) => {
                for ty in ts.iter().chain(us.iter()) {
                    self.discover_type(ty);
                }
            }

            Type::Product(ts) => {
                for ty in ts.iter() {
                    self.discover_type(ty);
                }
            }

            Type::Range(lo, hi) => {
                self.in_types.extend([*lo, *hi]);
            }

            Type::Number => {}

            Type::Invalid => {}
        }
    }
}

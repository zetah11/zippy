use std::cmp::Ordering;
use std::collections::HashMap;

use zippy_common::message::Span;
use zippy_common::mir::{
    Block, Branch, BranchNode, Context, Decls, Statement, StmtNode, Type, TypeId, Types, Value,
    ValueDef, ValueNode,
};
use zippy_common::names::{Name, Names};

pub fn flatten(names: &mut Names, types: &mut Types, context: &mut Context, decls: Decls) -> Decls {
    let mut flattener = Flattener::new(names, types, context);
    flattener.flatten_decls(decls)
}

#[derive(Debug)]
struct Flattener<'a> {
    names: &'a mut Names,
    types: &'a mut Types,
    context: &'a mut Context,

    mapping: HashMap<Name, Vec<Name>>,
    environment: HashMap<Name, Value>,
}

impl<'a> Flattener<'a> {
    pub fn new(names: &'a mut Names, types: &'a mut Types, context: &'a mut Context) -> Self {
        Self {
            names,
            types,
            context,

            mapping: HashMap::new(),
            environment: HashMap::new(),
        }
    }

    pub fn flatten_decls(&mut self, decls: Decls) -> Decls {
        assert!(decls.values.is_empty());
        assert!(decls.functions.is_empty());

        let mut defs = Vec::with_capacity(decls.defs.len());
        for def in decls.defs {
            let bind = self.flatten_def(&def.name, def.bind);
            defs.push(ValueDef {
                bind,
                name: def.name,
                span: def.span,
            });
        }

        Decls::new(defs)
    }

    fn flatten_def(&mut self, name: &Name, bind: Block) -> Block {
        let ty = self.context.get(name);
        let ty = self.flatten_type(&ty);
        self.context.replace(*name, ty);

        let mut exprs = Vec::with_capacity(bind.stmts.len());

        for expr in bind.stmts {
            let node = match expr.node {
                StmtNode::Tuple { name, values } => {
                    let values: Vec<_> = values
                        .into_iter()
                        .flat_map(|value| self.flatten_value(value))
                        .collect();
                    let mut names = Vec::with_capacity(values.len());
                    for value in values {
                        let new_name = self.names.fresh(expr.span, name);
                        self.context.add(new_name, value.ty);

                        names.push(new_name);
                        self.environment.insert(new_name, value);
                    }

                    self.mapping.insert(name, names);
                    continue;
                }

                StmtNode::Proj { name, of, at } => {
                    let tuple = match self.mapping.get(&of) {
                        Some(tuples) => tuples,
                        None => unreachable!(),
                    };

                    let value = match tuple.get(at) {
                        Some(name) => match self.environment.get(name) {
                            Some(value) => value.clone(),
                            None => Value {
                                node: ValueNode::Name(*name),
                                span: expr.span,
                                ty: self.context.get(name),
                            },
                        },
                        None => unreachable!(),
                    };

                    self.environment.insert(name, value);
                    continue;
                }

                StmtNode::Function { name, params, body } => {
                    let mut new_params = Vec::with_capacity(params.len());
                    for param in params {
                        new_params.extend(self.flatten_param(expr.span, param));
                    }

                    new_params.shrink_to_fit();
                    let body = self.flatten_def(&name, body);

                    StmtNode::Function {
                        name,
                        params: new_params,
                        body,
                    }
                }

                StmtNode::Apply { names, fun, args } => {
                    let args = args
                        .into_iter()
                        .flat_map(|arg| self.flatten_value(arg))
                        .collect();

                    let fun = self.flatten_name(fun);
                    let names = names
                        .into_iter()
                        .flat_map(|name| self.flatten_param(expr.span, name))
                        .collect();

                    StmtNode::Apply { names, fun, args }
                }

                StmtNode::Join { name, param, body } => {
                    let body = self.flatten_def(&name, body);
                    StmtNode::Join { name, param, body }
                }
            };

            let ty = self.flatten_type(&expr.ty);
            exprs.push(Statement { node, ty, ..expr })
        }

        let branch = {
            let node = match bind.branch.node {
                BranchNode::Jump(to, arg) => {
                    let to = self.flatten_name(to);
                    let arg = {
                        let mut res = self.flatten_value(arg);
                        assert!(res.len() == 1);
                        res.remove(0)
                    };
                    BranchNode::Jump(to, arg)
                }

                BranchNode::Return(values) => {
                    let res = values
                        .into_iter()
                        .flat_map(|value| self.flatten_value(value))
                        .collect();
                    BranchNode::Return(res)
                }
            };

            let ty = self.flatten_type(&bind.branch.ty);
            Branch {
                node,
                ty,
                ..bind.branch
            }
        };

        exprs.shrink_to_fit();

        let ty = self.flatten_type(&bind.ty);

        Block {
            stmts: exprs,
            branch,
            ty,
            ..bind
        }
    }

    fn flatten_name(&self, name: Name) -> Name {
        match self.environment.get(&name) {
            Some(Value {
                node: ValueNode::Name(name),
                ..
            }) => self.flatten_name(*name),

            Some(_) | None => name,
        }
    }

    fn flatten_name_value(&mut self, name: Name) -> Value {
        match self.environment.get(&name) {
            Some(Value {
                node: ValueNode::Name(name),
                ..
            }) => self.flatten_name_value(*name),

            Some(val) => val.clone(),

            None => {
                let ty = self.context.get(&name);
                let ty = self.flatten_type(&ty);
                let span = self.names.get_span(&name);
                Value {
                    node: ValueNode::Name(name),
                    span,
                    ty,
                }
            }
        }
    }

    fn flatten_param(&mut self, at: Span, name: Name) -> Vec<Name> {
        let ty = self.context.get(&name);
        match self.types.get(&ty) {
            Type::Product(ts) => {
                let mut names = Vec::with_capacity(ts.len());

                for t in ts.clone() {
                    let new_name = self.names.fresh(at, name);
                    self.context.add(new_name, t);

                    names.extend(self.flatten_param(at, new_name));
                }

                self.mapping.insert(name, names.clone());
                names
            }

            Type::Invalid | Type::Fun(..) | Type::Range(..) => vec![name],
        }
    }

    fn flatten_value(&mut self, value: Value) -> Vec<Value> {
        let node = match value.node {
            ValueNode::Name(name) => match (self.mapping.get(&name), self.environment.get(&name)) {
                (Some(names), None) => {
                    return names
                        .clone()
                        .into_iter()
                        .map(|name| self.flatten_name_value(name))
                        .collect()
                }
                (None, Some(value)) => value.node.clone(),
                (None, None) => ValueNode::Name(name),
                (Some(_), Some(_)) => unreachable!(),
            },
            ValueNode::Num(i) => ValueNode::Num(i),
            ValueNode::Invalid => ValueNode::Invalid,
        };

        let ty = self.flatten_type(&value.ty);

        vec![Value { node, ty, ..value }]
    }

    fn flatten_type(&mut self, ty: &TypeId) -> TypeId {
        let ts = self.flatten_types(ty);

        match ts.len().cmp(&1) {
            Ordering::Equal => ts[0],
            Ordering::Greater => self.types.add(Type::Product(ts)),
            Ordering::Less => unreachable!(),
        }
    }

    fn flatten_types(&mut self, ty: &TypeId) -> Vec<TypeId> {
        match self.types.get(ty) {
            Type::Range(..) => vec![*ty],
            Type::Product(ts) => ts
                .clone()
                .iter()
                .flat_map(|t| self.flatten_types(t))
                .collect(),
            Type::Fun(ts, us) => {
                let us = us.clone();
                let ts = ts
                    .clone()
                    .iter()
                    .flat_map(|t| self.flatten_types(t))
                    .collect();
                let us = us.iter().flat_map(|u| self.flatten_types(u)).collect();
                vec![self.types.add(Type::Fun(ts, us))]
            }
            Type::Invalid => vec![*ty],
        }
    }
}

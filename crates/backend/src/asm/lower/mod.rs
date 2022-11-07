mod location;

use std::collections::HashMap;

use log::{debug, trace};

use common::names::Name;
use common::{lir, mir};

use location::Location;

pub fn lower(
    entry: Option<Name>,
    types: &mir::Types,
    context: &mir::Context,
    decls: mir::Decls,
) -> lir::Program {
    debug!("lowering mir to lir");
    assert!(decls.defs.is_empty());
    let mut lowerer = Lowerer::new(entry, decls, types, context);
    lowerer.lower();

    lir::Program {
        procs: lowerer.procs,
        values: lowerer.values,
        types: lowerer.types,
        context: lowerer.context,
        info: lowerer.info,
    }
}

struct Lowerer<'a> {
    worklist: Vec<Name>,
    decls: mir::Decls,

    procs: HashMap<Name, lir::Procedure>,
    values: HashMap<Name, lir::Value>,
    types: lir::Types,
    context: lir::Context,
    info: lir::NameInfo,

    old_types: &'a mir::Types,
    old_context: &'a mir::Context,

    names: HashMap<Name, Location>,
    virtual_id: usize,
}

impl<'a> Lowerer<'a> {
    pub fn new(
        entry: Option<Name>,
        decls: mir::Decls,
        types: &'a mir::Types,
        context: &'a mir::Context,
    ) -> Self {
        trace!("discovery");
        let worklist = mir::discover(entry, &decls);
        let mut res = Self {
            worklist,
            decls,

            procs: HashMap::new(),
            values: HashMap::new(),
            types: lir::Types::new(),
            context: lir::Context::new(),
            info: lir::NameInfo::new(),

            old_types: types,
            old_context: context,

            names: HashMap::new(),
            virtual_id: 0,
        };

        res.find_globals();
        res
    }

    pub fn lower(&mut self) {
        trace!("lowering context");
        self.lower_context();

        trace!("{} names left to lower", self.worklist.len());
        while let Some(name) = self.worklist.pop() {
            if let Some((params, body)) = self.decls.functions.remove(&name) {
                let proc = self.lower_function(params, body);
                self.procs.insert(name, proc);
                self.info.add(name, lir::Info::procedure());
            } else if let Some(value) = self.decls.values.remove(&name) {
                let value = self.lower_value(&mut Vec::new(), value);
                self.values.insert(name, value);
                self.info.add(name, lir::Info::constant());
            }

            trace!("{} names left to lower", self.worklist.len());
        }
    }

    fn lower_context(&mut self) {
        for (name, ty) in self.old_context.iter() {
            let ty = self.lower_type(*ty);
            self.context.add(*name, ty);
        }
    }

    fn find_globals(&mut self) {
        for name in self.worklist.iter() {
            if self.decls.functions.contains_key(name) || self.decls.values.contains_key(name) {
                self.names.insert(*name, Location::Global);
            }
        }
    }

    fn lower_function(&mut self, params: Vec<Name>, body: mir::ExprSeq) -> lir::Procedure {
        let new_params: Vec<_> = params
            .iter()
            .map(|name| {
                let ty = self.context.get(name);
                self.fresh_reg(ty)
            })
            .collect();

        let moved_params: Vec<_> = params
            .into_iter()
            .map(|param| self.name_to_reg(param))
            .collect();

        let mut builder = lir::ProcBuilder::new_without_continuations(new_params.clone());

        let ret = builder.fresh_id();
        builder.add_continuations(vec![ret]);

        let mut exits = vec![];

        let mut block_param = vec![];
        let mut insts = vec![];
        let mut id = builder.fresh_id();

        let entry = id;

        for (param, target) in new_params.into_iter().zip(moved_params) {
            insts.push(lir::Instruction::Copy(
                lir::Target::Register(target),
                lir::Value::Register(param),
            ));
        }

        for expr in body.exprs {
            match expr.node {
                mir::ExprNode::Apply { names, fun, args } => {
                    let new_args: Vec<_> = args
                        .iter()
                        .map(|arg| {
                            let ty = self.lower_type(arg.ty);
                            self.fresh_reg(ty)
                        })
                        .collect();

                    let args: Vec<_> = args
                        .into_iter()
                        .map(|arg| self.lower_value(&mut insts, arg))
                        .collect();

                    for (arg, value) in new_args.iter().copied().zip(args) {
                        insts.push(lir::Instruction::Copy(lir::Target::Register(arg), value));
                    }

                    let fun = self.name_to_value(fun);

                    let cont = builder.fresh_id();

                    self.add_block(
                        &mut builder,
                        id,
                        block_param,
                        insts,
                        lir::Branch::Call(fun, new_args, vec![cont]),
                    );

                    block_param = names
                        .into_iter()
                        .map(|name| {
                            let ty = self.context.get(&name);
                            let name = self.name_to_reg(name);
                            (name, ty)
                        })
                        .collect();

                    insts = Vec::new();
                    id = cont;
                }

                mir::ExprNode::Tuple { name, values } => {
                    let values = values
                        .into_iter()
                        .map(|value| self.lower_value(&mut insts, value))
                        .collect();
                    let name = self.name_to_target(name);
                    insts.push(lir::Instruction::Tuple(name, values));
                }

                mir::ExprNode::Proj { name, of, at } => {
                    let ty = self.context.get(&name);

                    let index = self.types.offsetof(&ty, at);
                    let of = self.name_to_value(of);
                    let name = self.name_to_target(name);

                    insts.push(lir::Instruction::Index(name, of, index));
                }

                mir::ExprNode::Join { .. } => todo!(),

                mir::ExprNode::Function { .. } => unreachable!(),
            }
        }

        match body.branch.node {
            mir::BranchNode::Return(values) => {
                let new_args: Vec<_> = values
                    .iter()
                    .map(|value| {
                        let ty = self.lower_type(value.ty);
                        self.fresh_reg(ty)
                    })
                    .collect();

                let values: Vec<_> = values
                    .into_iter()
                    .map(|value| self.lower_value(&mut insts, value))
                    .collect();

                for (arg, value) in new_args.iter().copied().zip(values) {
                    insts.push(lir::Instruction::Copy(lir::Target::Register(arg), value));
                }

                let branch = lir::Branch::Return(ret, new_args);

                self.add_block(&mut builder, id, block_param, insts, branch);

                exits.push(id);
            }

            mir::BranchNode::Jump(..) => todo!(),
        }

        builder.build(entry, exits)
    }

    fn add_block(
        &mut self,
        builder: &mut lir::ProcBuilder,
        id: lir::BlockId,
        params: Vec<(lir::Register, lir::TypeId)>,
        mut instructions: Vec<lir::Instruction>,
        branch: lir::Branch,
    ) {
        let params = params
            .into_iter()
            .map(|(reg, ty)| {
                let new_param = self.fresh_reg(ty);
                instructions.insert(
                    0,
                    lir::Instruction::Copy(
                        lir::Target::Register(reg),
                        lir::Value::Register(new_param),
                    ),
                );
                new_param
            })
            .collect();

        builder.add(id, params, instructions, branch);
    }

    fn lower_value(&mut self, within: &mut Vec<lir::Instruction>, value: mir::Value) -> lir::Value {
        match value.node {
            mir::ValueNode::Invalid => {
                within.push(lir::Instruction::Crash);
                lir::Value::Integer(0)
            }

            mir::ValueNode::Int(i) => lir::Value::Integer(i),

            mir::ValueNode::Name(name) => match self.names.get(&name).unwrap() {
                Location::Local(id, ty) => {
                    lir::Value::Register(lir::Register::Virtual(lir::Virtual { id: *id, ty: *ty }))
                }

                Location::Global => lir::Value::Name(name),
            },
        }
    }

    fn lower_type(&mut self, ty: mir::TypeId) -> lir::TypeId {
        match self.old_types.get(&ty) {
            mir::Type::Range(lo, hi) => self.types.add(lir::Type::Range(*lo, *hi)),

            mir::Type::Invalid => self.types.add(lir::Type::Range(0, 0)),

            mir::Type::Product(ts) => {
                let ts = ts.iter().map(|t| self.lower_type(*t)).collect();
                self.types.add(lir::Type::Product(ts))
            }

            mir::Type::Fun(t, u) => {
                let t = t.iter().copied().map(|t| self.lower_type(t)).collect();
                let u = u.iter().copied().map(|u| self.lower_type(u)).collect();
                self.types.add(lir::Type::Fun(t, u))
            }
        }
    }

    fn name_to_value(&mut self, name: Name) -> lir::Value {
        match self.names.get(&name).unwrap() {
            Location::Local(id, ty) => {
                lir::Value::Register(lir::Register::Virtual(lir::Virtual { id: *id, ty: *ty }))
            }

            Location::Global => lir::Value::Name(name),
        }
    }

    fn name_to_target(&mut self, name: Name) -> lir::Target {
        if let Some(loc) = self.names.get(&name) {
            match loc {
                Location::Local(id, ty) => {
                    lir::Target::Register(lir::Register::Virtual(lir::Virtual { id: *id, ty: *ty }))
                }

                Location::Global => lir::Target::Name(name),
            }
        } else {
            let reg = self.name_to_reg(name);
            lir::Target::Register(reg)
        }
    }

    fn fresh_reg(&mut self, ty: lir::TypeId) -> lir::Register {
        let id = lir::Virtual {
            id: self.virtual_id,
            ty,
        };
        self.virtual_id += 1;
        lir::Register::Virtual(id)
    }

    fn name_to_reg(&mut self, name: Name) -> lir::Register {
        let ty = self.context.get(&name);

        let id = self.virtual_id;
        let reg = lir::Virtual { id, ty };

        self.virtual_id += 1;

        self.names.insert(name, Location::Local(id, ty));

        lir::Register::Virtual(reg)
    }
}

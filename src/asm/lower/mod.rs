use std::collections::HashMap;

use crate::lir;
use crate::mir;
use crate::resolve::names::Name;

pub fn lower(entry: Option<Name>, decls: mir::Decls) -> lir::Program {
    assert!(decls.defs.is_empty());
    let mut lowerer = Lowerer::new(entry, decls);
    lowerer.lower();

    lir::Program {
        procs: lowerer.procs,
        values: lowerer.values,
    }
}

struct Lowerer {
    worklist: Vec<Name>,
    decls: mir::Decls,

    procs: HashMap<Name, lir::Proc>,
    values: HashMap<Name, lir::Global>,
}

impl Lowerer {
    pub fn new(entry: Option<Name>, decls: mir::Decls) -> Self {
        let worklist = mir::discover(entry, &decls);
        Self {
            worklist,
            decls,
            procs: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn lower(&mut self) {
        while let Some(name) = self.worklist.pop() {
            if let Some((param, body)) = self.decls.functions.remove(&name) {
                let proc = self.lower_function(param, body);
                self.procs.insert(name, proc);
            } else if let Some(value) = self.decls.values.remove(&name) {
                let value = self.lower_global(value);
                self.values.insert(name, value);
            }
        }
    }

    fn lower_function(&mut self, _param: Name, body: mir::ExprSeq) -> lir::Proc {
        let mut builder = lir::ProcBuilder::default();

        let entry = {
            let mut insts = Vec::new();

            for expr in body.exprs {
                match expr.node {
                    mir::ExprNode::Apply { name, fun, arg } => {
                        let arg = self.lower_value(&mut insts, arg);
                        let fun = self.name_to_value(fun);
                        let name = self.name_to_target(name);

                        insts.extend([
                            lir::Inst::Push(arg),
                            lir::Inst::Call(fun),
                            lir::Inst::Pop(name),
                        ]);
                    }

                    mir::ExprNode::Tuple { .. } => todo!(),
                    mir::ExprNode::Proj { .. } => todo!(),

                    mir::ExprNode::Join { .. } => todo!(),

                    mir::ExprNode::Function { .. } => unreachable!(),
                }
            }

            let branch = match body.branch.node {
                mir::BranchNode::Return(value) => {
                    let value = self.lower_value(&mut insts, value);
                    lir::Branch::Return(value)
                }

                mir::BranchNode::Jump(..) => todo!(),
            };

            builder.add(lir::Block { insts, branch })
        };

        builder.build(entry, entry)
    }

    fn lower_value(&mut self, within: &mut Vec<lir::Inst>, value: mir::Value) -> lir::Value {
        match value.node {
            mir::ValueNode::Int(i) => lir::Value::Integer(i),
            mir::ValueNode::Invalid => {
                within.push(lir::Inst::Crash);
                lir::Value::Integer(0)
            }

            mir::ValueNode::Name(_) => todo!(),
        }
    }

    fn lower_global(&mut self, value: mir::Value) -> lir::Global {
        match value.node {
            mir::ValueNode::Int(i) => lir::Global {
                size: 1,
                value: vec![i],
            },

            mir::ValueNode::Invalid => lir::Global {
                size: 0,
                value: vec![],
            },

            mir::ValueNode::Name(_) => todo!(),
        }
    }

    fn name_to_value(&mut self, _name: Name) -> lir::Value {
        todo!()
    }

    fn name_to_target(&mut self, _name: Name) -> lir::Target {
        todo!()
    }
}

use common::message::Span;
use common::mir::{Branch, BranchNode, Statement, StmtNode, TypeId, Value, ValueNode};
use common::names::Name;

use super::action::Action;
use super::environment::Env;
use super::value::{Operation, ReducedValue};
use super::Interpreter;

pub struct ReduceResult {
    pub action: Action,
    pub values: Option<Vec<ReducedValue>>,
    pub operation: Option<Operation>,
}

impl Interpreter<'_> {
    pub fn reduce_value(&self, value: &Value) -> Option<ReducedValue> {
        match &value.node {
            ValueNode::Int(_) | ValueNode::Invalid => Some(ReducedValue::Static(value.clone())),

            ValueNode::Name(name) if self.globals.contains_key(name) => self
                .globals
                .get(name)
                .map(|val| ReducedValue::Static(val.clone())),

            ValueNode::Name(name) => self.frames.last()?.get(name).cloned(),
        }
    }

    pub fn reduce_op(&mut self, op: Operation, args: Vec<ReducedValue>) -> ReduceResult {
        match op {
            Operation::Branch(branch) => self.reduce_branch(branch, args),
            Operation::Statement(stmt) => self.reduce_stmt(stmt, args),
        }
    }

    fn reduce_branch(&mut self, branch: Branch, args: Vec<ReducedValue>) -> ReduceResult {
        match branch.node {
            BranchNode::Jump(..) => todo!(),
            BranchNode::Return(unreduced_args) => {
                self.reduce_return((unreduced_args, args), branch.span, branch.ty)
            }
        }
    }

    fn reduce_stmt(&mut self, stmt: Statement, args: Vec<ReducedValue>) -> ReduceResult {
        match stmt.node {
            StmtNode::Function { .. } => todo!(),
            StmtNode::Join { .. } => todo!(),
            StmtNode::Tuple { .. } => todo!(),
            StmtNode::Proj { .. } => todo!(),

            StmtNode::Apply {
                names,
                fun,
                args: unreduced_args,
            } => self.reduce_call(names, fun, (unreduced_args, args), stmt.span, stmt.ty),
        }
    }

    /// Partially evaluate a return. This will always produce an operation (the
    /// least a function can do is return :)
    fn reduce_return(
        &mut self,
        args: (Vec<Value>, Vec<ReducedValue>),
        span: Span,
        ty: TypeId,
    ) -> ReduceResult {
        let mut inst_args = Vec::new();
        let mut values = Vec::new();

        for (arg, old) in args.1.into_iter().zip(args.0) {
            let (value, arg) = match arg {
                ReducedValue::Static(value) => (ReducedValue::Static(value.clone()), value),
                ReducedValue::Dynamic(_) => (arg, old),
            };

            values.push(value);
            inst_args.push(arg);
        }

        let branch = Branch {
            node: BranchNode::Return(inst_args),
            span,
            ty,
        };

        ReduceResult {
            action: Action::Exit {
                return_values: values,
            },
            values: None,
            operation: Some(Operation::Branch(branch)),
        }
    }

    /// Partially evaluate a call. If the provided function is unknown, the
    /// instruction is left mostly as is. Otherwise, a call is made. If all of
    /// the arguments (including the function) are statically known and all
    /// effects are handled, no instruction will be produced.
    fn reduce_call(
        &mut self,
        names: Vec<Name>,
        fun: Name,
        mut args: (Vec<Value>, Vec<ReducedValue>),
        span: Span,
        ty: TypeId,
    ) -> ReduceResult {
        let (new_fun, fun_ty, static_fun) = match args.1.remove(0) {
            ReducedValue::Static(Value {
                node: ValueNode::Name(fun),
                ty,
                ..
            }) => (fun, ty, true),
            ReducedValue::Dynamic(Value {
                node: ValueNode::Name(fun),
                ty,
                ..
            }) => (fun, ty, false),
            _ => unreachable!(),
        };

        let (push_inst, action) = if let Some(place) = self.place_of(&new_fun) {
            let params = self.functions.get(&fun).unwrap();
            assert_eq!(params.len(), args.1.len());

            let mut new_env = Env::new();

            // If this is a pure function and all of its arguments have been provided, then
            // the function will fully reduce.
            let is_dynamic = !self.types.is_pure(&fun_ty)
                || args.1.iter().any(|arg| arg.is_dynamic())
                || !static_fun;

            for (param, arg) in params.iter().zip(args.1.iter()) {
                let arg = match arg {
                    ReducedValue::Static(value) => value.clone(),
                    ReducedValue::Dynamic(value) => value.clone(),
                };

                let arg = ReducedValue::Dynamic(arg);

                new_env.add(*param, arg);
            }

            let action = Action::Enter {
                place,
                env: new_env,
                return_names: names.clone(),
            };

            (is_dynamic, action)
        } else {
            (true, Action::None)
        };

        let inst = if push_inst {
            let mut inst_args = Vec::new();

            for (arg, old) in args.1.into_iter().zip(args.0) {
                if let ReducedValue::Static(arg) = arg {
                    inst_args.push(arg);
                } else {
                    inst_args.push(old);
                }
            }

            let fun = if static_fun { new_fun } else { fun };

            Some(Statement {
                node: StmtNode::Apply {
                    names,
                    fun,
                    args: inst_args,
                },
                span,
                ty,
            })
        } else {
            None
        };

        ReduceResult {
            action,
            operation: inst.map(Operation::Statement),
            values: None,
        }
    }
}

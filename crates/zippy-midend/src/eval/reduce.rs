use zippy_common::message::Span;
use zippy_common::mir::{Branch, BranchNode, Statement, StmtNode, TypeId, Value, ValueNode};
use zippy_common::names::Name;
use zippy_common::Driver;

use super::action::Action;
use super::environment::Env;
use super::value::{Operation, ReducedValue};
use super::Interpreter;

pub struct ReduceResult {
    pub action: Action,
    pub values: Option<Vec<ReducedValue>>,
    pub operation: Option<Operation>,
}

impl<D: Driver> Interpreter<'_, D> {
    pub fn reduce_value(&self, value: &Value) -> Option<ReducedValue> {
        match &value.node {
            ValueNode::Num(_) | ValueNode::Invalid => {
                Some(self.locally_static_value(value.clone()))
            }

            ValueNode::Name(name) if self.globals.contains_key(name) => self
                .globals
                .get(name)
                .map(|val| self.locally_static_value(val.clone())),

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

            StmtNode::Coerce { name, of, from, to } => {
                let mut args = args;
                self.reduce_coerce(name, (args.remove(0), of), from, to, stmt.span)
            }

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
            let (value, arg) = if arg.is_static(self.frame_index()) {
                (self.locally_static_value(arg.value.clone()), arg.value)
            } else {
                (arg, old)
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
        let new_fun = args.1.remove(0);
        let static_fun = new_fun.is_static(self.frame_index());
        let (new_fun, fun_ty) = match new_fun.value {
            Value {
                node: ValueNode::Name(fun),
                ty,
                ..
            } => (fun, ty),
            _ => unreachable!(),
        };

        let (push_inst, action) = match (self.place_of(&new_fun), self.functions.get(&new_fun)) {
            (Some(place), Some(params)) => {
                assert_eq!(params.len(), args.1.len());

                let mut new_env = Env::new();

                // If this is a pure function and all of its arguments have been provided, then
                // the function will fully reduce.
                let is_dynamic = !self.types.is_pure(&fun_ty)
                    || args.1.iter().any(|arg| arg.is_dynamic(self.frame_index()))
                    || !static_fun;

                for (param, arg) in params.iter().zip(args.1.iter()) {
                    new_env.add(*param, arg.clone());
                }

                let action = Action::Enter {
                    place,
                    env: new_env,
                    return_names: names.clone(),
                };

                (is_dynamic, action)
            }

            _ => (true, Action::None),
        };

        let inst = if push_inst {
            let mut inst_args = Vec::new();

            for (arg, old) in args.1.into_iter().zip(args.0) {
                if arg.is_static(self.frame_index()) {
                    inst_args.push(arg.value);
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

    /// Reduce a coercion.
    fn reduce_coerce(
        &mut self,
        name: Name,
        of: (ReducedValue, Name),
        from: TypeId,
        into: TypeId,
        span: Span,
    ) -> ReduceResult {
        let reduced = of.0;
        let unreduced = of.1;

        // TODO: check if the reduced value actually fits for this.

        let operation = if reduced.is_dynamic(self.frame_index()) {
            Some(Operation::Statement(Statement {
                node: StmtNode::Coerce {
                    name,
                    of: unreduced,
                    from,
                    to: into,
                },
                span,
                ty: into,
            }))
        } else {
            match reduced.value.node {
                ValueNode::Name(of) => Some(Operation::Statement(Statement {
                    node: StmtNode::Coerce {
                        name,
                        of,
                        from,
                        to: into,
                    },
                    span,
                    ty: into,
                })),

                _ => None,
            }
        };

        ReduceResult {
            action: Action::None,
            operation,
            values: Some(vec![reduced]),
        }
    }
}

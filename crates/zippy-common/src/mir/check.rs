use crate::message::Span;
use crate::names::Names;

use super::pretty::Prettier;
use super::{
    Block, BranchNode, Context, Decls, Statement, StmtNode, Type, TypeId, Types, Value, ValueNode,
};

pub fn check(names: &Names, types: &Types, context: &Context, decls: &Decls) -> bool {
    let mut checker = MirChecker::new(names, types, context);
    checker.check_decls(decls);
    checker.error
}

struct MirChecker<'a> {
    names: &'a Names,
    types: &'a Types,
    context: &'a Context,
    error: bool,
}

impl<'a> MirChecker<'a> {
    pub fn new(names: &'a Names, types: &'a Types, context: &'a Context) -> Self {
        Self {
            names,
            types,
            context,
            error: false,
        }
    }

    pub fn check_decls(&mut self, decls: &Decls) {
        for def in decls.defs.iter() {
            let expected = self.context.get(&def.name);
            self.check_type(def.span, expected, def.bind.ty);

            self.check_exprs(&def.bind);
        }
    }

    fn check_exprs(&mut self, exprs: &Block) {
        let retty = exprs.ty;

        for expr in exprs.stmts.iter() {
            self.check_expr(expr);
        }

        match &exprs.branch.node {
            BranchNode::Return(values) => {
                if values.len() == 1 {
                    self.check_value(retty, &values[0]);
                } else {
                    match self.types.get(&retty) {
                        Type::Product(ts) => {
                            assert!(values.len() == ts.len());
                            for (value, ty) in values.iter().zip(ts.iter()) {
                                self.check_value(*ty, value);
                            }
                        }

                        Type::Invalid => {}

                        _ => {
                            assert!(values.len() == 1);
                            self.check_value(retty, &values[0]);
                        }
                    }
                }
            }

            BranchNode::Jump(..) => todo!(),
        }
    }

    fn check_expr(&mut self, expr: &Statement) {
        match &expr.node {
            StmtNode::Join { .. } => todo!(),
            StmtNode::Function { name, params, body } => {
                let ty = self.context.get(name);
                match self.types.get(&ty) {
                    Type::Fun(t, u) => {
                        assert!(t.len() == params.len());
                        for (param, t) in params.iter().zip(t.iter()) {
                            let other_t = self.context.get(param);
                            self.check_type(expr.span, *t, other_t);
                        }

                        if u.len() == 1 {
                            self.check_type(body.span, u[0], body.ty);
                        } else {
                            match self.types.get(&body.ty) {
                                Type::Product(ts) => {
                                    assert!(u.len() == ts.len());
                                    for (u, t) in u.iter().zip(ts.iter()) {
                                        self.check_type(body.span, *u, *t);
                                    }
                                }

                                Type::Invalid => {}

                                _ => {
                                    assert!(u.len() == 1);
                                    self.check_type(body.span, u[0], body.ty);
                                }
                            }
                        }
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }

                self.check_exprs(body);
            }

            StmtNode::Apply { names, fun, args } => {
                let ty = self.context.get(fun);
                match self.types.get(&ty) {
                    Type::Fun(t, u) => {
                        assert!(t.len() == args.len());
                        for (arg, t) in args.iter().zip(t.iter()) {
                            self.check_value(*t, arg);
                        }

                        assert!(u.len() == names.len());
                        for (name, u) in names.iter().zip(u.iter()) {
                            let other_u = self.context.get(name);
                            self.check_type(expr.span, *u, other_u);
                        }
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }
            }

            StmtNode::Tuple { name, values } => {
                let ty = self.context.get(name);
                match self.types.get(&ty) {
                    Type::Product(ts) => {
                        assert!(values.len() == ts.len());
                        for (value, t) in values.iter().zip(ts.iter()) {
                            self.check_value(*t, value);
                        }
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }
            }

            StmtNode::Proj { name, of, at } => {
                let ty = self.context.get(of);
                match self.types.get(&ty) {
                    Type::Product(ts) => {
                        let other_ty = self.context.get(name);
                        match ts.get(*at) {
                            Some(t) => self.check_type(expr.span, other_ty, *t),
                            None => panic!("index out of range"),
                        }
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }
            }
        }
    }

    fn check_value(&mut self, expected: TypeId, value: &Value) {
        let actual = value.ty;
        self.check_type(value.span, expected, actual);

        match &value.node {
            ValueNode::Int(_) => match self.types.get(&actual) {
                Type::Range(..) | Type::Invalid => {}
                _ => unreachable!(),
            },

            ValueNode::Name(name) => {
                let actual2 = self.context.get(name);
                self.check_type(value.span, actual, actual2);
            }

            ValueNode::Invalid => {}
        }
    }

    fn check_type(&mut self, at: Span, expected: TypeId, actual: TypeId) {
        match (self.types.get(&expected), self.types.get(&actual)) {
            (Type::Invalid, _) | (_, Type::Invalid) => {}
            _ => {
                if actual != expected {
                    self.error = true;
                    let prettier = Prettier::new(self.names, self.types);
                    eprintln!("type mismatch at {:?}", at);
                    eprintln!(
                        "expected {}, got {}",
                        prettier.pretty_type(&expected),
                        prettier.pretty_type(&actual)
                    );
                }
            }
        }
    }
}

use crate::message::Span;
use crate::resolve::names::Names;

use super::pretty::Prettier;
use super::{
    BranchNode, Context, Decls, Expr, ExprNode, ExprSeq, Type, TypeId, Types, Value, ValueNode,
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

    fn check_exprs(&mut self, exprs: &ExprSeq) {
        let retty = exprs.ty;

        for expr in exprs.exprs.iter() {
            self.check_expr(expr);
        }

        match &exprs.branch.node {
            BranchNode::Return(value) => self.check_value(retty, value),

            BranchNode::Jump(..) => todo!(),
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match &expr.node {
            ExprNode::Join { .. } => todo!(),
            ExprNode::Function { name, param, body } => {
                let ty = self.context.get(name);
                match self.types.get(&ty) {
                    Type::Fun(t, u) => {
                        let other_t = self.context.get(param);
                        self.check_type(expr.span, *t, other_t);
                        self.check_type(body.span, *u, body.ty);
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }

                self.check_exprs(body);
            }

            ExprNode::Apply { name, fun, arg } => {
                let ty = self.context.get(fun);
                match self.types.get(&ty) {
                    Type::Fun(t, u) => {
                        let other_u = self.context.get(name);
                        self.check_value(*t, arg);
                        self.check_type(expr.span, *u, other_u);
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }
            }

            ExprNode::Tuple { name, values } => {
                let ty = self.context.get(name);
                match self.types.get(&ty) {
                    Type::Product(t, u) => {
                        assert!(values.len() == 2);
                        self.check_value(*t, &values[0]);
                        self.check_value(*u, &values[1]);
                    }

                    Type::Invalid => {}

                    _ => unreachable!(),
                }
            }

            ExprNode::Proj { name, of, at } => {
                let ty = self.context.get(of);
                match self.types.get(&ty) {
                    Type::Product(t, u) => {
                        let other_ty = self.context.get(name);
                        match at {
                            0 => self.check_type(expr.span, other_ty, *t),
                            1 => self.check_type(expr.span, other_ty, *u),
                            _ => panic!("projection too big"),
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

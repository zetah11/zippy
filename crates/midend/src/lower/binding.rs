use common::message::Span;
use common::mir::{
    Block, Branch, BranchNode, Statement, StmtNode, TypeId, Value, ValueDef, ValueNode,
};
use common::names::Name;

use super::{HiPat, HiPatNode, Inst, Lowerer};

impl Lowerer<'_> {
    /// Turn a monomorphic and possibly destructuring binding like
    ///
    /// ```z
    /// let a: 10, b: 10 = 1, 2
    /// ```
    ///
    /// into several simple disjoint bindings like
    ///
    /// ```z
    /// let _t1: 10 * 10 = 1, 2
    /// let a: 10 = _t1.0
    /// let b: 10 = _t1.1
    /// ```
    pub fn destruct_monomorphic(
        &mut self,
        inst: &Inst,
        ctx: Name,
        span: Span,
        pat: HiPat,
        bind: Block,
    ) {
        match pat.node {
            HiPatNode::Name(name) => {
                let ty = self.lower_type(inst, pat.data);
                self.context.add(name, ty);
                self.values.push(ValueDef { name, span, bind });
            }

            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(inst, pat.data);
                let value = self.fresh_name(a.span + b.span, ctx, ty);

                self.values.push(ValueDef {
                    name: value,
                    span,
                    bind,
                });

                self.bind_projection(inst, ctx, value, 0, *a);
                self.bind_projection(inst, ctx, value, 1, *b);
            }

            // The typechecker should remove all annotations
            HiPatNode::Anno(..) => unreachable!(),

            HiPatNode::Wildcard | HiPatNode::Invalid => {}
        }
    }

    /// Turn a local pattern binding into a series of simple bindings of the form `let <name> = <expr>`. Returns the
    /// name this pattern gets replaced with, and a expressions needed to bind the pattern itself. This list should be
    /// appended *after* the name of the pattern is bound.
    pub fn destruct_local(&mut self, inst: &Inst, ctx: Name, pat: HiPat) -> (Name, Vec<Statement>) {
        let mut after = Vec::new();
        let (name, _) = self.bind_local(inst, ctx, &mut after, pat);
        after.reverse();

        (name, after)
    }

    /// The same as [`Self::destruct_local`], except the "followup" expressions will be inserted into `after` in reverse
    /// order.
    fn bind_local(
        &mut self,
        inst: &Inst,
        ctx: Name,
        after: &mut Vec<Statement>,
        pat: HiPat,
    ) -> (Name, TypeId) {
        let ty = self.lower_type(inst, pat.data);

        let name = match pat.node {
            HiPatNode::Name(name) => {
                self.context.add(name, ty);
                name
            }

            HiPatNode::Tuple(a, b) => {
                let a_span = a.span;
                let b_span = b.span;

                let (b, b_ty) = self.bind_local(inst, ctx, after, *b);
                let (a, a_ty) = self.bind_local(inst, ctx, after, *a);

                let name = self.fresh_name(pat.span, ctx, ty);

                let a_bind = StmtNode::Proj {
                    name: a,
                    of: name,
                    at: 0,
                };
                let b_bind = StmtNode::Proj {
                    name: b,
                    of: name,
                    at: 1,
                };

                let a_bind = Statement {
                    ty: a_ty,
                    span: a_span,
                    node: a_bind,
                };
                let b_bind = Statement {
                    ty: b_ty,
                    span: b_span,
                    node: b_bind,
                };

                after.push(b_bind);
                after.push(a_bind);

                name
            }

            // The typechecker should remove all annotations.
            HiPatNode::Anno(..) => unreachable!(),

            HiPatNode::Wildcard | HiPatNode::Invalid => self.fresh_name(pat.span, ctx, ty),
        };

        (name, ty)
    }

    /// Bind the pattern `pat` to the projection (i.e. the tuple element) of `of` at index `at`.
    fn bind_projection(&mut self, inst: &Inst, ctx: Name, of: Name, at: usize, pat: HiPat) {
        let span = pat.span;
        let ty = self.lower_type(inst, pat.data.clone());
        let target = self.fresh_name(pat.span, ctx, ty);

        let binding = Statement {
            ty,
            span,
            node: StmtNode::Proj {
                name: target,
                of,
                at,
            },
        };

        let ret = Value {
            ty,
            span,
            node: ValueNode::Name(target),
        };

        let ret = Branch {
            ty,
            span,
            node: BranchNode::Return(vec![ret]),
        };

        let bind = Block {
            ty,
            span,
            exprs: vec![binding],
            branch: ret,
        };

        self.destruct_monomorphic(inst, ctx, span, pat, bind);
    }
}

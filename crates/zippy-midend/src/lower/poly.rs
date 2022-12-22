use std::collections::HashMap;

use zippy_common::message::Span;
use zippy_common::mir::ValueDef;
use zippy_common::names::Name;

use super::{HiExpr, HiExprNode, HiPat, HiPatNode, HiType, Inst, Lowerer};

impl Lowerer<'_> {
    pub fn instantiate(
        &mut self,
        at: Span,
        inst: &Inst,
        name: &Name,
        args: Vec<(Span, HiType)>,
    ) -> Name {
        let template = self.templates.get(name).unwrap();
        assert_eq!(template.implicits.len(), args.len());

        let mut inst = inst.clone();
        for ((name, _), (_, ty)) in template.implicits.iter().zip(args) {
            inst.insert(*name, ty);
        }

        let anno = template.anno.clone();
        let bind = template.bind.clone(); // ouch

        let ty = self.lower_type(&inst, anno);
        let target = self.fresh_name(at, *name, ty);

        let mut name_map = HashMap::new();
        let bind = self.copy_expr(&mut name_map, name, &target, bind);
        let bind = self.lower_expr(&inst, target, bind);

        self.values.push(ValueDef {
            span: at,
            name: target,
            bind,
        });

        target
    }

    /// Create a copy of the definition with `span` and `bind` for the instantiation with name `Name`.
    fn copy_expr(
        &mut self,
        name_map: &mut HashMap<Name, Name>,
        old_name: &Name,
        new_name: &Name,
        bind: HiExpr,
    ) -> HiExpr {
        let node = match bind.node {
            node @ (HiExprNode::Num(_) | HiExprNode::Hole | HiExprNode::Invalid) => node,

            HiExprNode::Name(name) => {
                if let Some(new_name) = name_map.get(&name) {
                    HiExprNode::Name(*new_name)
                } else {
                    HiExprNode::Name(name)
                }
            }

            HiExprNode::Tuple(a, b) => {
                let a = Box::new(self.copy_expr(name_map, old_name, new_name, *a));
                let b = Box::new(self.copy_expr(name_map, old_name, new_name, *b));
                HiExprNode::Tuple(a, b)
            }

            HiExprNode::App(fun, arg) => {
                let fun = Box::new(self.copy_expr(name_map, old_name, new_name, *fun));
                let arg = Box::new(self.copy_expr(name_map, old_name, new_name, *arg));
                HiExprNode::App(fun, arg)
            }

            HiExprNode::Lam(param, body) => {
                let param = self.copy_pat(name_map, old_name, new_name, param);
                let body = Box::new(self.copy_expr(name_map, old_name, new_name, *body));
                HiExprNode::Lam(param, body)
            }

            HiExprNode::Inst(body, args) => {
                let body = Box::new(self.copy_expr(name_map, old_name, new_name, *body));
                HiExprNode::Inst(body, args)
            }

            HiExprNode::Anno(expr, span, ty) => {
                let expr = Box::new(self.copy_expr(name_map, old_name, new_name, *expr));
                HiExprNode::Anno(expr, span, ty)
            }

            HiExprNode::Coerce(expr, id) => {
                let expr = Box::new(self.copy_expr(name_map, old_name, new_name, *expr));
                HiExprNode::Coerce(expr, id)
            }
        };

        HiExpr {
            node,
            span: bind.span,
            data: bind.data, // `instantiate` will take care of instantiating the type correctly
        }
    }

    fn copy_pat(
        &mut self,
        name_map: &mut HashMap<Name, Name>,
        old_name: &Name,
        new_name: &Name,
        pat: HiPat,
    ) -> HiPat {
        let node = match pat.node {
            node @ (HiPatNode::Wildcard | HiPatNode::Invalid) => node,

            HiPatNode::Name(name) => {
                let copied_name = self.names.rebase(&name, old_name, new_name);
                name_map.insert(name, copied_name);

                HiPatNode::Name(copied_name)
            }

            HiPatNode::Tuple(a, b) => {
                let a = Box::new(self.copy_pat(name_map, old_name, new_name, *a));
                let b = Box::new(self.copy_pat(name_map, old_name, new_name, *b));
                HiPatNode::Tuple(a, b)
            }

            HiPatNode::Anno(pat, ty) => {
                let pat = Box::new(self.copy_pat(name_map, old_name, new_name, *pat));
                HiPatNode::Anno(pat, ty)
            }

            HiPatNode::Coerce(pat, id) => {
                let pat = Box::new(self.copy_pat(name_map, old_name, new_name, *pat));
                HiPatNode::Coerce(pat, id)
            }
        };

        HiPat {
            node,
            span: pat.span,
            data: pat.data, // `instantiate` will take care of instantiate the type correctly
        }
    }
}

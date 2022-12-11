//! Constructs the dependency graph of the program. A name `n` depends on a name
//! `m` if `m` occurs in the definition of `n`. This considers dependencies
//! between both value- and type names.

use std::collections::{HashMap, HashSet};

use zippy_common::names::Name;
use zippy_common::thir::{Decls, Expr, ExprNode, Pat, PatNode, Type, TypeDef, ValueDef};

#[derive(Debug, Default)]
pub struct Dependencies {
    deps: HashMap<Name, HashSet<Name>>,
}

impl Dependencies {
    pub fn find(decls: &Decls) -> HashMap<Name, HashSet<Name>> {
        let mut finder = Self::default();

        for def in decls.types.iter() {
            finder.search_type(def);
        }

        for def in decls.values.iter() {
            finder.search_value(def);
        }

        finder.deps
    }

    fn search_type(&mut self, def: &TypeDef) {
        let (defined, refers) = self.pat_defines(&def.pat);

        let in_anno = self.type_refers(&defined, &def.anno);
        let in_bind = self.type_refers(&defined, &def.bind);
        let res: HashSet<_> = refers.into_iter().chain(in_anno).chain(in_bind).collect();

        for name in defined {
            self.deps
                .entry(name)
                .or_default()
                .extend(res.iter().copied());
        }
    }

    fn search_value(&mut self, def: &ValueDef) {
        let (defined, refers) = self.pat_defines(&def.pat);
        let shadowed = defined
            .iter()
            .chain(def.implicits.iter().map(|(name, _)| name))
            .copied()
            .collect();

        let in_anno = self.type_refers(&shadowed, &def.anno);
        let in_bind = self.expr_refers(&shadowed, &def.bind);
        let res: HashSet<_> = refers.into_iter().chain(in_anno).chain(in_bind).collect();

        for name in defined {
            self.deps
                .entry(name)
                .or_default()
                .extend(res.iter().copied());
        }
    }

    /// Find all the names defined by the pattern, as well as those referred to
    /// by it.
    fn pat_defines(&self, pat: &Pat) -> (HashSet<Name>, HashSet<Name>) {
        match &pat.node {
            PatNode::Invalid | PatNode::Wildcard => Default::default(),
            PatNode::Anno(pat, ty) => {
                let in_anno = self.type_refers(&HashSet::new(), ty);
                let (defined, in_pat) = self.pat_defines(pat);
                let refs = in_anno.union(&in_pat).copied().collect();
                (defined, refs)
            }

            PatNode::Tuple(a, b) => {
                let (def_a, ref_a) = self.pat_defines(a);
                let (def_b, ref_b) = self.pat_defines(b);
                let defs = def_a.union(&def_b).copied().collect();
                let refs = ref_a.union(&ref_b).copied().collect();
                (defs, refs)
            }

            PatNode::Name(name) => (HashSet::from([*name]), HashSet::new()),
        }
    }

    fn type_refers(&self, shadowed: &HashSet<Name>, ty: &Type) -> HashSet<Name> {
        match ty {
            Type::Invalid | Type::Number | Type::Var(..) | Type::Type => HashSet::new(),
            Type::Range(..) => HashSet::new(),

            Type::Name(name) if shadowed.contains(name) => HashSet::new(),
            Type::Name(name) => HashSet::from([*name]),

            Type::Fun(t, u) | Type::Product(t, u) => {
                let t = self.type_refers(shadowed, t);
                let u = self.type_refers(shadowed, u);
                t.into_iter().chain(u).collect()
            }

            Type::Instantiated(..) => unreachable!(),
        }
    }

    fn expr_refers(&self, shadowed: &HashSet<Name>, ex: &Expr) -> HashSet<Name> {
        match &ex.node {
            ExprNode::Invalid | ExprNode::Int(_) | ExprNode::Hole => HashSet::new(),

            ExprNode::Name(name) if shadowed.contains(name) => HashSet::new(),
            ExprNode::Name(name) => HashSet::from([*name]),

            ExprNode::Anno(ex, _, ty) => {
                let ex = self.expr_refers(shadowed, ex);
                let ty = self.type_refers(shadowed, ty);
                ex.into_iter().chain(ty).collect()
            }

            ExprNode::App(x, y) | ExprNode::Tuple(x, y) => {
                let x = self.expr_refers(shadowed, x);
                let y = self.expr_refers(shadowed, y);
                x.into_iter().chain(y).collect()
            }

            ExprNode::Inst(ex, ties) => {
                let mut ex = self.expr_refers(shadowed, ex);
                for (_, ty) in ties.iter() {
                    ex.extend(self.type_refers(shadowed, ty));
                }

                ex
            }

            ExprNode::Lam(pat, body) => {
                let (defined, refers) = self.pat_defines(pat);
                let refers = refers.difference(shadowed).copied();
                let shadowed = shadowed.union(&defined).copied().collect();
                let body = self.expr_refers(&shadowed, body);

                refers.chain(body).collect()
            }
        }
    }
}

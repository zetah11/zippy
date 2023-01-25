//! Constructs the dependency graph of the program. A name `n` depends on a name
//! `m` if `m` occurs in the definition of `n`. This considers dependencies
//! between both value- and type names.

use std::collections::{HashMap, HashSet};

use zippy_common::hir2::{Decls, Expr, ExprNode, Pat, PatNode, Type, TypeDef, ValueDef};
use zippy_common::names2::Name;

/// A [`DefIndex`] is an index into either the `values` or the `types` list in a
/// [`Decls`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DefIndex {
    Value(usize),
    Type(usize),
}

#[derive(Debug, Default)]
pub struct Dependencies {
    deps: HashMap<Name, HashSet<Name>>,
    map: HashMap<Name, DefIndex>,
}

impl Dependencies {
    pub fn find(decls: &Decls) -> HashMap<DefIndex, HashSet<DefIndex>> {
        let mut finder = Self::default();

        for (index, def) in decls.types.iter().enumerate() {
            finder.search_type(def, index);
        }

        for (index, def) in decls.values.iter().enumerate() {
            finder.search_value(def, index);
        }

        finder
            .deps
            .into_iter()
            .map(|(key, values)| {
                let key = finder.map.get(&key).unwrap();
                let values = values
                    .into_iter()
                    .map(|name| finder.map.get(&name).unwrap())
                    .copied()
                    .collect();

                (*key, values)
            })
            .collect()
    }

    fn search_type(&mut self, def: &TypeDef, index: usize) {
        let (defined, refers) = pat_defines(&def.pat);

        let in_anno = type_refers(&defined, &def.anno);
        let in_bind = type_refers(&defined, &def.bind);
        let res: HashSet<_> = refers.into_iter().chain(in_anno).chain(in_bind).collect();

        for name in defined {
            self.deps
                .entry(name)
                .or_default()
                .extend(res.iter().copied());

            self.map.insert(name, DefIndex::Type(index));
        }
    }

    fn search_value(&mut self, def: &ValueDef, index: usize) {
        let (defined, refers) = pat_defines(&def.pat);
        let shadowed = defined
            .iter()
            .chain(def.implicits.iter().map(|(name, _)| name))
            .copied()
            .collect();

        let in_anno = type_refers(&shadowed, &def.anno);
        let in_bind = expr_refers(&shadowed, &def.bind);
        let res: HashSet<_> = refers.into_iter().chain(in_anno).chain(in_bind).collect();

        for name in defined {
            self.deps
                .entry(name)
                .or_default()
                .extend(res.iter().copied());

            self.map.insert(name, DefIndex::Value(index));
        }
    }
}

/// Find all the names defined by the pattern, as well as those referred to
/// by it.
fn pat_defines(pat: &Pat) -> (HashSet<Name>, HashSet<Name>) {
    match &pat.node {
        PatNode::Invalid | PatNode::Wildcard => Default::default(),

        PatNode::Coerce(pat, _) => pat_defines(pat),

        PatNode::Anno(pat, ty) => {
            let in_anno = type_refers(&HashSet::new(), ty);
            let (defined, in_pat) = pat_defines(pat);
            let refs = in_anno.union(&in_pat).copied().collect();
            (defined, refs)
        }

        PatNode::Tuple(a, b) => {
            let (def_a, ref_a) = pat_defines(a);
            let (def_b, ref_b) = pat_defines(b);
            let defs = def_a.union(&def_b).copied().collect();
            let refs = ref_a.union(&ref_b).copied().collect();
            (defs, refs)
        }

        PatNode::Name(name) => (HashSet::from([*name]), HashSet::new()),
    }
}

fn type_refers(shadowed: &HashSet<Name>, ty: &Type) -> HashSet<Name> {
    match ty {
        Type::Invalid | Type::Number | Type::Var(..) | Type::Type => HashSet::new(),
        Type::Range(..) => HashSet::new(),

        Type::Name(name) if shadowed.contains(name) => HashSet::new(),
        Type::Name(name) => HashSet::from([*name]),

        Type::Fun(t, u) | Type::Product(t, u) => {
            let t = type_refers(shadowed, t);
            let u = type_refers(shadowed, u);
            t.into_iter().chain(u).collect()
        }

        Type::Instantiated(..) => unreachable!(),
    }
}

fn expr_refers(shadowed: &HashSet<Name>, ex: &Expr) -> HashSet<Name> {
    match &ex.node {
        ExprNode::Invalid | ExprNode::Num(_) | ExprNode::Hole => HashSet::new(),

        ExprNode::Name(name) if shadowed.contains(name) => HashSet::new(),
        ExprNode::Name(name) => HashSet::from([*name]),

        ExprNode::Anno(ex, _, ty) => {
            let ex = expr_refers(shadowed, ex);
            let ty = type_refers(shadowed, ty);
            ex.into_iter().chain(ty).collect()
        }

        ExprNode::Coerce(ex, _) => expr_refers(shadowed, ex),

        ExprNode::App(x, y) | ExprNode::Tuple(x, y) => {
            let x = expr_refers(shadowed, x);
            let y = expr_refers(shadowed, y);
            x.into_iter().chain(y).collect()
        }

        ExprNode::Inst(ex, ties) => {
            let mut ex = expr_refers(shadowed, ex);
            for (_, ty) in ties.iter() {
                ex.extend(type_refers(shadowed, ty));
            }

            ex
        }

        ExprNode::Lam(pat, body) => {
            let (defined, refers) = pat_defines(pat);
            let refers = refers.difference(shadowed).copied();
            let shadowed = shadowed.union(&defined).copied().collect();
            let body = expr_refers(&shadowed, body);

            refers.chain(body).collect()
        }
    }
}

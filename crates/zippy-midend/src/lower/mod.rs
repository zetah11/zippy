mod binding;
mod expr;
mod poly;
mod types;

use std::collections::HashMap;

use log::{debug, trace};

use zippy_common::message::{Messages, Span};
use zippy_common::mir::{Context, Decls, TypeId, Types, ValueDef};
use zippy_common::names::{Name, Names};
use zippy_common::thir::{self, UniVar};
use zippy_common::Driver;

type HiType = thir::Type;
type HiPat = thir::Pat<HiType>;
type HiPatNode = thir::PatNode<HiType>;
type HiExpr = thir::Expr<HiType>;
type HiExprNode = thir::ExprNode<HiType>;
type HiValueDef = thir::ValueDef<HiType>;
type HiDecls = thir::Decls<HiType>;

type Inst = HashMap<Name, HiType>;

pub fn lower(
    driver: &mut impl Driver,
    subst: &HashMap<UniVar, (HashMap<Name, HiType>, HiType)>,
    names: &mut Names,
    decls: HiDecls,
) -> (Types, Context, Decls) {
    debug!("beginning lowering");
    let mut lowerer = Lowerer::new(names, subst);

    let ctx = lowerer.names.root();
    let decls = lowerer.lower_decls(ctx, decls);

    driver.report(lowerer.messages);

    trace!("done lowering");

    (lowerer.types, lowerer.context, decls)
}

#[derive(Debug)]
struct Lowerer<'a> {
    types: Types,
    names: &'a mut Names,
    subst: &'a HashMap<UniVar, (HashMap<Name, HiType>, HiType)>,
    messages: Messages,
    context: Context,
    templates: HashMap<Name, HiValueDef>,

    values: Vec<ValueDef>,
}

impl<'a> Lowerer<'a> {
    fn new(
        names: &'a mut Names,
        subst: &'a HashMap<UniVar, (HashMap<Name, HiType>, HiType)>,
    ) -> Self {
        Self {
            types: Types::new(),
            names,
            subst,
            messages: Messages::new(),
            context: Context::new(),
            templates: HashMap::new(),

            values: Vec::new(),
        }
    }

    fn lower_decls(&mut self, ctx: Name, decls: HiDecls) -> Decls {
        let mut monomorphic = Vec::new();

        for def in decls.values {
            if def.implicits.is_empty() {
                monomorphic.push(def);
            } else {
                match def.pat.node {
                    HiPatNode::Name(name) => {
                        self.templates.insert(name, def);
                    }

                    _ => todo!("polymorphic destruction"),
                }
            }
        }

        let inst = HashMap::new();
        for def in monomorphic {
            let bind = self.lower_expr(&inst, ctx, def.bind);
            self.destruct_monomorphic(&inst, ctx, def.span, def.pat, bind);
        }

        Decls::new(self.values.drain(..).collect())
    }

    fn fresh_name(&mut self, at: Span, ctx: Name, ty: TypeId) -> Name {
        let name = self.names.fresh(at, ctx);
        self.context.add(name, ty);
        name
    }
}
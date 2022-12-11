mod bind;
mod check;
//mod components;
mod dependency;
mod infer;
mod lower;
mod solve;
mod types;

use log::{info, trace};

use zippy_common::message::Messages;
use zippy_common::names::{Name, Names};
use zippy_common::thir::{
    Because, Constraint, Context, Decls, Expr, ExprNode, Mutability, Pat, PatNode, Type,
    TypeckResult, ValueDef,
};
use zippy_common::{hir, Driver};

use solve::Unifier;

pub fn typeck(driver: &mut impl Driver, names: &Names, decls: hir::Decls<Name>) -> TypeckResult {
    info!("beginning typechecking");

    let mut typer = Typer::new(names);
    let decls = typer.lower(decls);

    let deps = dependency::Dependencies::find(&decls);
    for (name, deps) in deps {
        fn pretty(names: &Names, name: &Name) -> String {
            match &names.get_path(name).1 {
                zippy_common::names::Actual::Lit(s) => s.clone(),
                zippy_common::names::Actual::Generated(g) => g.to_string("_t"),
                zippy_common::names::Actual::Root => "root".into(),
                zippy_common::names::Actual::Scope(s) => s.to_string(),
            }
        }

        println!(
            "{}: [{}]",
            pretty(names, &name),
            deps.into_iter()
                .map(|name| pretty(names, &name))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let decls = typer.typeck(decls);

    info!("solving {} constraints", typer.constraints.len());
    typer.solve();

    typer.messages.merge(typer.unifier.messages);
    driver.report(typer.messages);

    trace!("done typechecking");

    TypeckResult {
        decls,
        context: typer.context,
        subst: typer.unifier.subst,
        constraints: typer.constraints,
    }
}

#[derive(Debug)]
struct Typer<'a> {
    pub messages: Messages,
    context: Context,
    unifier: Unifier<'a>,
    constraints: Vec<Constraint>,
}

impl<'a> Typer<'a> {
    pub fn new(names: &'a Names) -> Self {
        Self {
            messages: Messages::new(),
            context: Context::new(),
            unifier: Unifier::new(names),
            constraints: Vec::new(),
        }
    }

    pub fn typeck(&mut self, decls: Decls) -> Decls<Type> {
        let types = Vec::with_capacity(decls.types.len());
        for def in decls.types.into_iter() {
            let _kind = self.infer_type(&def.bind);
            todo!()
        }

        let mut pats = Vec::with_capacity(decls.values.len());
        for def in decls.values.iter() {
            pats.push(self.bind_def(def));
        }

        let mut values = Vec::with_capacity(decls.values.len());
        for (def, pat) in decls.values.into_iter().zip(pats) {
            let def = self.check_def(pat, def);
            values.push(def);
        }

        Decls { values, types }
    }

    pub fn solve(&mut self) {
        let mut len = self.constraints.len();

        while !self.constraints.is_empty() {
            let constraints: Vec<_> = self.constraints.drain(..).collect();

            for constraint in constraints {
                match constraint {
                    Constraint::IntType { at, because, ty } => {
                        let _ = self.int_type(at, because, ty);
                    }

                    Constraint::Assignable { at, into, from } => {
                        self.assignable(at, into, from);
                    }
                }
            }

            if self.constraints.len() >= len {
                match self.constraints.first().unwrap() {
                    Constraint::IntType { at, .. } => self.messages.at(*at).tyck_no_progress(),
                    Constraint::Assignable { at, .. } => self.messages.at(*at).tyck_no_progress(),
                };

                break;
            }

            len = self.constraints.len();
        }
    }

    fn bind_def(&mut self, def: &ValueDef) -> Pat<Type> {
        let pat = self.bind_generic(def.pat.clone(), &def.implicits, def.anno.clone());
        self.make_mutability(Mutability::Immutable, &pat);
        pat
    }

    fn check_def(&mut self, pat: Pat<Type>, def: ValueDef) -> ValueDef<Type> {
        self.make_mutability(Mutability::Mutable, &pat);
        let ty = pat.data.make_mutability(Mutability::Mutable);
        let bind = self.check(Because::Annotation(def.span), def.bind, ty);
        self.make_mutability(Mutability::Immutable, &pat);

        ValueDef {
            pat,
            span: def.span,
            implicits: def.implicits,
            anno: def.anno,
            bind,
        }
    }

    fn make_mutability<T>(&mut self, mutability: Mutability, pat: &Pat<T>) {
        match &pat.node {
            PatNode::Name(name) => self.context.make_mutability(name, mutability),
            PatNode::Anno(pat, _) => self.make_mutability(mutability, pat),
            PatNode::Tuple(x, y) => {
                self.make_mutability(mutability, x);
                self.make_mutability(mutability, y);
            }
            PatNode::Invalid | PatNode::Wildcard => {}
        }
    }

    fn pretty(&mut self, ty: &Type) -> String {
        self.unifier.pretty(ty)
    }
}

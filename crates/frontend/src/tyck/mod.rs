mod bind;
mod check;
mod infer;
mod lower;
mod solve;

use log::{info, trace};

use common::message::Messages;
use common::names::Name;
use common::thir::{
    Because, Constraint, Context, Decls, Expr, ExprNode, Pat, PatNode, Type, TypeckResult, ValueDef,
};
use common::{hir, Driver};

use solve::Unifier;

pub fn typeck(driver: &mut impl Driver, decls: hir::Decls<Name>) -> TypeckResult {
    info!("beginning typechecking");

    let mut typer = Typer::new();
    let decls = typer.lower(decls);
    let decls = typer.typeck(decls);
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

#[derive(Debug, Default)]
struct Typer {
    pub messages: Messages,
    context: Context,
    unifier: Unifier,
    constraints: Vec<Constraint>,
}

impl Typer {
    pub fn new() -> Self {
        Self {
            messages: Messages::new(),
            context: Context::new(),
            unifier: Unifier::new(),
            constraints: Vec::new(),
        }
    }

    pub fn typeck(&mut self, decls: Decls) -> Decls<Type> {
        let (pats, binds): (Vec<_>, Vec<_>) = decls
            .values
            .into_iter()
            .map(|decl| {
                (
                    (decl.pat, decl.anno.clone()),
                    (decl.span, decl.bind, decl.anno),
                )
            })
            .unzip();

        // Let's be extremely careful with our ordering now...

        let mut new_pats = Vec::with_capacity(pats.len());
        let mut new_binds = Vec::with_capacity(binds.len());

        for (pat, anno) in pats {
            new_pats.push(self.bind_pat(pat, anno));
        }

        for (span, bind, anno) in binds {
            new_binds.push((span, self.check(Because::Annotation(span), bind, anno)));
        }

        let values = new_pats
            .into_iter()
            .zip(new_binds.into_iter())
            .map(|(pat, (span, bind))| ValueDef {
                span,
                pat,
                anno: Type::Invalid,
                bind,
            })
            .collect();

        Decls { values }
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
                }
            }

            if self.constraints.len() >= len {
                match self.constraints.first().unwrap() {
                    Constraint::IntType { at, .. } => self.messages.at(*at).tykc_no_progress(),
                };

                break;
            }

            len = self.constraints.len();
        }
    }
}
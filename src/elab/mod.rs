mod eval;
mod flatten;
mod hoist;
mod lower;

use log::{debug, info, trace};

use crate::mir::{self, check};
use crate::resolve::names::{Name, Names};
use crate::tyck::TypeckResult;
use crate::{Driver, EvalAmount};

pub fn elaborate(
    driver: &mut impl Driver,
    names: &mut Names,
    tyckres: TypeckResult,
    entry: Option<Name>,
) -> (mir::Types, mir::Context, mir::Decls) {
    info!("beginning elaboration");

    let (mut types, mut context, res) = lower::lower(
        driver,
        &tyckres.subst,
        names,
        tyckres.context,
        tyckres.decls,
    );

    let mut error = check(names, &types, &context, &res);
    if error {
        eprintln!("error during lowering");
    } else {
        trace!("lowering is type-correct");
    }

    let res = flatten::flatten(names, &mut types, &mut context, res);
    if !error {
        error = check(names, &types, &context, &res);
        if error {
            eprintln!("error during flattening");
        } else {
            trace!("flattening is type-correct");
        }
    }

    let res = if driver.eval_amount() == EvalAmount::Full {
        let res = eval::evaluate(driver, &mut context, names, &types, res, entry);

        if !error {
            error = check(names, &types, &context, &res);
            if error {
                eprintln!("error during evaluation");
            } else {
                trace!("evaluation is type-correct");
            }
        }
        res
    } else {
        debug!("skipped evaluation");
        res
    };

    let res = hoist::hoist(driver, names, &mut context, res);
    if !error {
        error = check(names, &types, &context, &res);
        if error {
            eprintln!("error during hoisting");
        } else {
            trace!("hoisting is type-correct");
        }
    }

    trace!("done elaborating");

    (types, context, res)
}

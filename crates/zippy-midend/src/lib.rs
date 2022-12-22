mod eval;
mod flatten;
mod hoist;
mod lower;

use log::{debug, info, trace};

use zippy_common::mir::{self, check};
use zippy_common::names::{Name, Names};
use zippy_common::thir::TypeckResult;
use zippy_common::{Driver, EvalAmount};

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
        tyckres.coercions,
        tyckres.defs,
        names,
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

    let res = hoist::hoist(driver, names, &mut context, res);
    if !error {
        error = check(names, &types, &context, &res);
        if error {
            eprintln!("error during hoisting");
        } else {
            trace!("hoisting is type-correct");
        }
    }

    let res = match driver.eval_amount() {
        EvalAmount::Full => {
            let res = eval::evaluate(driver, &context, names, &types, entry, res);

            if !error {
                error = check(names, &types, &context, &res);
                if error {
                    eprintln!("error during evaluation");
                } else {
                    trace!("evaluation is type-correct");
                }
            }

            res
        }

        EvalAmount::Types | EvalAmount::None => {
            debug!("skipped evaluation");
            res
        }
    };

    trace!("done elaborating");

    (types, context, res)
}

mod eval;
mod hoist;
mod lower;

use log::{info, trace};

use crate::mir;
use crate::resolve::names::{Name, Names};
use crate::tyck::TypeckResult;
use crate::Driver;

pub fn elaborate(
    driver: &mut impl Driver,
    names: &mut Names,
    tyckres: TypeckResult,
    entry: Option<Name>,
) -> (mir::Types, mir::Decls) {
    info!("beginning elaboration");

    let (types, mut context, res) = lower::lower(
        driver,
        &tyckres.subst,
        names,
        tyckres.context,
        tyckres.decls,
    );
    let res = eval::evaluate(driver, &mut context, names, &types, res, entry);
    let res = hoist::hoist(driver, names, &mut context, res);

    trace!("done elaborating");

    (types, res)
}

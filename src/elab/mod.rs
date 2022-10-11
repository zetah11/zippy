mod eval;
mod lower;

use log::{info, trace};

use crate::mir;
use crate::resolve::names::Names;
use crate::tyck::TypeckResult;
use crate::Driver;

pub fn elaborate(
    driver: &mut impl Driver,
    names: &mut Names,
    tyckres: TypeckResult,
) -> (mir::Types, mir::Decls) {
    info!("beginning elaboration");

    let (types, res) = lower::lower(driver, &tyckres.subst, names, tyckres.decls);
    let res = eval::evaluate(driver, names, &types, res);

    trace!("done elaborating");

    (types, res)
}

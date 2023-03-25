mod apply;
mod bind;
mod bound;
mod constrained;
mod generate;
mod messages;
mod solve;
mod types;

pub(crate) use self::apply::apply;
pub(crate) use self::bind::{get_bound, Bound};
pub(crate) use self::generate::{constrain, FlatProgram};
pub(crate) use self::solve::{solve, Solution};
pub(crate) use self::types::{CoercionState, Coercions, Constraint, Type, UnifyVar};

use std::collections::HashMap;

use zippy_common::messages::{Message, Messages};
use zippy_common::source::Module;

use crate::{checked, Db};

pub fn check(
    db: &dyn Db,
    messages: &mut Vec<Message>,
    modules: impl Iterator<Item = Module>,
) -> checked::Program {
    let mut bound_modules = Vec::new();
    let mut context = HashMap::new();
    let mut counts = HashMap::new();

    let mut constraints = Vec::new();

    for module in modules {
        let bound = get_bound(db, module);
        constraints.extend(bound.constraints(db).iter().cloned());
        messages.extend(get_bound::accumulated::<Messages>(db, module));

        bound_modules.push(bound.module(db));
        context.extend(
            bound
                .context(db)
                .iter()
                .map(|(name, ty)| (*name, ty.clone())),
        );

        counts.extend(bound.counts(db).iter().map(|(span, count)| (*span, *count)));
    }

    let program = FlatProgram {
        modules: bound_modules,
        context,
        counts,
    };

    let constrained = constrain(program);
    constraints.extend(constrained.constraints);
    let mut solution = solve(db, constrained.counts, constraints);
    messages.append(&mut solution.messages);

    apply(constrained.program, solution)
}

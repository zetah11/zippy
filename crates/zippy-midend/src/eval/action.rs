use zippy_common::names::Name;

use super::environment::Env;
use super::place::Place;
use super::value::ReducedValue;

pub enum Action {
    Enter {
        place: Place,
        env: Env,
        return_names: Vec<Name>,
    },
    Exit {
        return_values: Vec<ReducedValue>,
    },
    None,
}

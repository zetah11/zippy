use std::str::FromStr;

use clap::error::ErrorKind;
use clap::CommandFactory;
use target_lexicon::Triple;

use crate::args::Arguments;

pub fn get_target(args: &Arguments) -> Triple {
    let target = match args.target {
        Some(ref target) => Triple::from_str(target),
        None => Ok(Triple::host()),
    };

    match target {
        Err(error) => {
            let error = error.to_string();
            let mut cmd = Arguments::command();
            cmd.error(ErrorKind::InvalidValue, error).exit()
        }

        Ok(target) => target,
    }
}

use zippy_common::names::{Actual, Name, Names};

pub fn mangle(names: &Names, name: &Name) -> String {
    let path = names.get_path(name);

    if path.0.is_none() {
        // Root name
        match &path.1 {
            Actual::Lit(name) => name.clone(),
            Actual::Generated(gen) => String::from(*gen),
            Actual::Root => unreachable!(),
            Actual::Scope(_) => unreachable!(),
        }
    } else {
        let mut path = path;
        let mut parts = vec![path.1.clone()];
        while let Some(ctx) = path.0 {
            path = names.get_path(&ctx);
            parts.push(path.1.clone());
        }

        let mut res = String::new();
        res.push('Z');

        for part in parts.into_iter().rev() {
            res.push_str(&match part {
                Actual::Lit(name) => format!("_n{name}"),
                Actual::Generated(id) => String::from(id),
                Actual::Scope(id) => format!("_{}", id),
                Actual::Root => String::new(),
            });
        }

        res
    }
}

use std::collections::HashMap;

use super::kinds::UniVar;

#[derive(Debug, Default)]
pub struct Namer {
    names: HashMap<UniVar, String>,
    count: usize,
}

impl Namer {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            count: 0,
        }
    }

    pub fn pretty(&mut self, var: UniVar) -> String {
        self.names
            .entry(var)
            .or_insert_with(|| {
                let count = self.count;
                self.count += 1;
                number_to_var_name(count)
            })
            .clone()
    }
}

fn number_to_var_name(mut num: usize) -> String {
    let mut result = String::new();
    result.push('\'');

    if num == 0 {
        result.push('A');
    }

    while num != 0 {
        result.push((b'A' + (num % 26) as u8) as char);
        num /= 26;
    }

    result.shrink_to_fit();
    result
}

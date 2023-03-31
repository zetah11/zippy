use std::collections::HashMap;

use super::instanced::{AbstractInstance, Instance, InstanceVar, Template, Type};
use super::Clarifier;

impl Clarifier {
    pub fn instantiate_template(&mut self, template: Template) -> Type {
        let Template { ty, exists } = template;

        let map = exists
            .into_iter()
            .map(|parameter| {
                let var = self.fresh_instance_var();
                (parameter, var)
            })
            .collect();

        self.instantiate_type(map, ty)
    }

    fn instantiate_type(&mut self, map: HashMap<AbstractInstance, InstanceVar>, ty: Type) -> Type {
        match ty {
            Type::Trait { instance } => {
                let instance = match instance {
                    Instance::Parameter(p) => match map.get(&p) {
                        Some(i) => Instance::Var(*i),
                        None => Instance::Parameter(p),
                    },

                    Instance::Var(v) => *self
                        .substitution
                        .get(&v)
                        .expect("ungeneralized instance var in type signature"),

                    i => i,
                };

                Type::Trait { instance }
            }

            Type::Range(range) => Type::Range(range),
            Type::Number => Type::Number,
            Type::Invalid(reason) => Type::Invalid(reason),
        }
    }
}

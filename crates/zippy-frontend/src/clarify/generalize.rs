use std::collections::HashSet;

use super::instanced::{Instance, InstanceVar, Item, ItemNode, Template, Type};
use super::Clarifier;

impl Clarifier {
    pub(super) fn generalize_item(&mut self, item: Item) -> Item {
        // Find free instance variables
        let names = item.names;
        let vars = match &item.node {
            ItemNode::Bound { body } => self.free_type_vars(&body.data),
            ItemNode::Let { pattern, .. } => self.free_type_vars(&pattern.data),
        };

        // Replace vars with new instance parameters
        // To avoid having to traverse the item *again*, we can just substitute
        // the unification variable with the appropriate abstract var since
        // `free_instance_vars` should never return an instance var which is
        // already substituted.
        let mut params = Vec::new();
        for var in vars {
            let abs = self.fresh_abstract_instance();
            assert!(self
                .substitution
                .insert(var, Instance::Parameter(abs))
                .is_none());
            params.push(abs);
        }

        // Adjust item templates
        for name in names.iter().copied() {
            let Template { exists, ty } = self
                .item_types
                .remove(&name)
                .expect("all items have been given a non-generalized template");

            assert!(exists.is_empty());

            let template = Template {
                exists: params.clone(),
                ty,
            };

            self.item_types.insert(name, template);
        }

        Item {
            names,
            node: item.node,
        }
    }

    fn free_type_vars(&self, ty: &Type) -> HashSet<InstanceVar> {
        match ty {
            Type::Trait { instance } => self.free_instance_vars(instance),
            Type::Range(_) | Type::Number | Type::Invalid(_) => HashSet::new(),
        }
    }

    fn free_instance_vars(&self, ins: &Instance) -> HashSet<InstanceVar> {
        match ins {
            Instance::Var(v) => match self.substitution.get(v) {
                Some(ins) => self.free_instance_vars(ins),
                None => HashSet::from([*v]),
            },

            Instance::Parameter(_) => HashSet::new(),
            Instance::Concrete(_) => HashSet::new(),
        }
    }
}

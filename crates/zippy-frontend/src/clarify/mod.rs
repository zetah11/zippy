//! To existentially clarify an item we need to do two things:
//!
//! - Create unique instances with appropriate dependencies for every entry
//! - Figure out which instance any given trait refers to, and generalize the
//!   instance variables where appropriate.
//!
//! Creating instances is done by figuring out which free ephemeral values are
//! needed by an entry. This can be done by traversing the syntax tree, passing
//! along every ephemeral name and returning every name referred. The set of
//! dependencies can then be computed by figuring out which ephemeral names are
//! referred to but declared in an outer scope.
//!
//! Figuring out the instances referred to by a trait can be done by assigning
//! each trait an "instance variable" and solving for the variables through
//! unification (generating the appropriate equations while traversing the
//! tree). Unconstrained instance variables which appear in the type of an item
//! can be generalized.

mod constrain;
mod generalize;
mod instanced;
mod instantiate;
mod messages;
mod unify;

use std::collections::{HashMap, HashSet};

use zippy_common::messages::{Message, MessageMaker};
use zippy_common::names::{ItemName, LocalName, Name};
use zippy_common::source::Span;

use self::instanced::InstanceIndex;
use crate::checked::{self, ItemIndex};
use crate::dependencies::get_dependencies;
use crate::Db;

pub fn clarify(db: &dyn Db, messages: &mut Vec<Message>, program: checked::Program) {
    let deps = get_dependencies(db, &program);

    let mut clarifier = Clarifier::new(program);
    clarifier.clarify_locals();

    for items in deps.order {
        clarifier.clarify_component(items);
    }

    messages.extend(clarifier.messages);
}

struct Clarifier {
    checked_program: checked::Program,

    items: HashMap<ItemIndex, instanced::Item>,
    instances: HashMap<InstanceIndex, instanced::EntryInstance>,

    item_types: HashMap<ItemName, instanced::Template>,
    local_types: HashMap<LocalName, instanced::Type>,

    equations: Vec<(Span, instanced::Type, instanced::Type)>,
    substitution: HashMap<instanced::InstanceVar, instanced::Instance>,

    abstract_counter: usize,
    index_counter: usize,
    var_counter: usize,

    messages: Vec<Message>,
}

impl Clarifier {
    pub fn new(program: checked::Program) -> Self {
        Self {
            checked_program: program,
            items: HashMap::new(),
            instances: HashMap::new(),
            item_types: HashMap::new(),
            local_types: HashMap::new(),
            equations: Vec::new(),
            substitution: HashMap::new(),

            abstract_counter: 0,
            index_counter: 0,
            var_counter: 0,

            messages: Vec::new(),
        }
    }

    /// Clarify the type of every local in the program.
    pub fn clarify_locals(&mut self) {
        let locals: Vec<_> = self.checked_program.local_types.drain().collect();
        for (name, ty) in locals {
            let ty = self.fresh_type(ty);
            assert!(self.local_types.insert(name, ty).is_none());
        }
    }

    /// Simultaneously clarify the existentials in a set of strongly connected
    /// items.
    pub fn clarify_component(&mut self, indicies: HashSet<checked::ItemIndex>) {
        let items: Vec<_> = indicies
            .iter()
            .map(|index| {
                let item = self
                    .checked_program
                    .items
                    .remove(index)
                    .expect("all item indicies are bound");
                (*index, item)
            })
            .collect();

        // Bind temporary templates for every name in the items.
        for (_, item) in items.iter() {
            for name in item.names.iter().copied() {
                let template = self
                    .checked_program
                    .item_types
                    .remove(&name)
                    .expect("all items have a type");
                let template = self.fresh_template(template);
                assert!(self.item_types.insert(name, template).is_none());
            }
        }

        // Generate equations
        let items: Vec<_> = items
            .into_iter()
            .map(|(index, item)| (index, self.constrain_item(item)))
            .collect();

        // Solve them
        self.solve();

        for (index, item) in items {
            let item = self.generalize_item(item);
            assert!(self.items.insert(index, item).is_none());
        }
    }

    fn solve(&mut self) {
        let equations: Vec<_> = self.equations.drain(..).collect();

        for (at, left, right) in equations {
            self.unify(at, left, right);
        }
    }

    /// Instantiate the type of `name`. Returns `None` if the name does not have
    /// a type.
    fn instantiate(&mut self, name: Name) -> Option<instanced::Type> {
        match name {
            Name::Item(name) => {
                let template = self.item_types.get(&name)?.clone();
                Some(self.instantiate_template(template))
            }

            Name::Local(name) => self.local_types.get(&name).cloned(),
        }
    }

    fn equate(&mut self, at: Span, left: instanced::Type, right: instanced::Type) {
        self.equations.push((at, left, right));
    }

    fn equate_trait(&mut self, at: Span, ty: instanced::Type, with: instanced::Instance) {
        match ty {
            instanced::Type::Trait { instance, .. } => {
                self.equate_instances(at, instance, with);
            }

            instanced::Type::Invalid(_) => {}

            _ => unreachable!("non-trait type on an entry expression"),
        }
    }

    /// Create a template with fresh instance variables.
    fn fresh_template(&mut self, template: checked::Template) -> instanced::Template {
        let checked::Template { ty } = template;

        // Existential variables are only created after generalization.
        instanced::Template {
            ty: self.fresh_type(ty),
            exists: Vec::new(),
        }
    }

    /// Create a type with fresh instance variables.
    fn fresh_type(&mut self, ty: checked::Type) -> instanced::Type {
        match ty {
            checked::Type::Trait { values: _ } => {
                let instance = instanced::Instance::Var(self.fresh_instance_var());
                instanced::Type::Trait { instance }
            }

            checked::Type::Range(range) => instanced::Type::Range(range),
            checked::Type::Number => instanced::Type::Number,
            checked::Type::Invalid(reason) => instanced::Type::Invalid(reason),
        }
    }

    fn fresh_abstract_instance(&mut self) -> instanced::AbstractInstance {
        let id = instanced::AbstractInstance(self.abstract_counter);
        self.abstract_counter += 1;
        id
    }

    fn fresh_instance_var(&mut self) -> instanced::InstanceVar {
        let id = instanced::InstanceVar(self.var_counter);
        self.var_counter += 1;
        id
    }

    fn fresh_instance_index(&mut self) -> instanced::InstanceIndex {
        let id = instanced::InstanceIndex(self.index_counter);
        self.index_counter += 1;
        id
    }

    fn at(&mut self, at: Span) -> MessageMaker<&mut Vec<Message>> {
        MessageMaker::new(&mut self.messages, at)
    }
}

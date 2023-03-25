mod flow;
mod instantiate;
mod types;

use std::collections::HashMap;

use zippy_common::messages::{Message, MessageMaker};
use zippy_common::names::ItemName;
use zippy_common::source::Span;

use crate::messages::TypeMessages;
use crate::resolved::Alias;
use crate::Db;

use self::types::NumericResult;
use super::constrained::DelayedConstraint;
use super::types::{CoercionState, CoercionVar, Coercions, Constraint, Template, Type, UnifyVar};

#[derive(Debug)]
pub struct Solution {
    pub messages: Vec<Message>,
    pub substitution: HashMap<UnifyVar, Type>,
    pub coercions: Coercions,
    pub aliases: HashMap<Alias, ItemName>,
    pub delayed: Vec<DelayedConstraint>,
}

pub fn solve(db: &dyn Db, counts: HashMap<Span, usize>, constraints: Vec<Constraint>) -> Solution {
    let mut solver = Solver::new(db, counts, constraints);
    solver.solve();

    Solution {
        messages: solver.messages,
        substitution: solver.substitution,
        coercions: solver.coercions,
        aliases: solver
            .aliases
            .into_iter()
            .flat_map(|(name, (item, _))| Some((name, item?)))
            .collect(),
        delayed: solver.delayed,
    }
}

struct Solver<'db> {
    db: &'db dyn Db,
    messages: Vec<Message>,
    counts: HashMap<Span, usize>,

    aliases: HashMap<Alias, (Option<ItemName>, Template)>,

    constraints: Vec<Constraint>,
    numeric: Vec<(Span, Type)>,
    textual: Vec<(Span, Type)>,
    type_numeric: Vec<(Span, Type)>,
    unitlike: Vec<(Span, Type)>,

    coercions: Coercions,
    substitution: HashMap<UnifyVar, Type>,
    delayed: Vec<DelayedConstraint>,
}

impl<'db> Solver<'db> {
    pub fn new(
        db: &'db dyn Db,
        counts: HashMap<Span, usize>,
        constraints: Vec<Constraint>,
    ) -> Self {
        Self {
            db,
            messages: Vec::new(),
            counts,

            aliases: HashMap::new(),

            constraints,
            numeric: Vec::new(),
            textual: Vec::new(),
            type_numeric: Vec::new(),
            unitlike: Vec::new(),

            coercions: Coercions::new(),
            substitution: HashMap::new(),
            delayed: Vec::new(),
        }
    }

    pub fn solve(&mut self) {
        while !self.constraints.is_empty() {
            let constraints: Vec<_> = self.constraints.drain(..).collect();
            let before = constraints.len();

            for constraint in constraints {
                self.solve_constraint(constraint);
            }

            let after = self.constraints.len();

            if after >= before {
                let constraints: Vec<_> = self.constraints.drain(..).collect();
                for constraint in constraints {
                    self.report_unsolvable(constraint);
                }
                break;
            }
        }

        self.solve_type_numerics();
        self.solve_checks();

        if !self.constraints.is_empty() {
            let constraints: Vec<_> = self.constraints.drain(..).collect();
            for constraint in constraints {
                self.report_unsolvable(constraint);
            }
        }
    }

    fn solve_constraint(&mut self, constraint: Constraint) {
        match constraint {
            Constraint::Alias {
                at,
                alias,
                of,
                name,
            } => self.alias(at, alias, of, name),

            Constraint::Assignable { at, id, into, from } => self.assign(at, id, into, from),

            Constraint::Equal(at, t, u) => self.equate(at, t, u),

            Constraint::Field {
                at,
                target,
                of,
                field,
            } => self.field(at, target, of, field),

            Constraint::Instantiated(at, ty, template) => self.instantiated(at, ty, template),
            Constraint::InstantiatedAlias(at, ty, alias) => self.instantiated_alias(at, ty, alias),

            Constraint::UnitLike(at, ty) => self.unitlike.push((at, ty)),
            Constraint::Textual(at, ty) => self.textual.push((at, ty)),
            Constraint::Numeric(at, ty) => self.numeric.push((at, ty)),
            Constraint::TypeNumeric(at, ty) => self.type_numeric.push((at, ty)),
        }
    }

    /// Check type literal validities.
    fn solve_checks(&mut self) {
        // Numeric
        let constraints: Vec<_> = self.numeric.drain(..).collect();
        for (at, ty) in constraints {
            match self.numeric(at, ty) {
                NumericResult::Ok => {}
                NumericResult::Unsolved(at, _) => self.at(at).ambiguous(),
                NumericResult::Error(messages) => self.messages.extend(messages),
            }
        }

        // Textual
        let constraints: Vec<_> = self.textual.drain(..).collect();
        for (at, ty) in constraints {
            self.textual(at, ty);
        }

        // Unitlike
        let constraints: Vec<_> = self.unitlike.drain(..).collect();
        for (at, ty) in constraints {
            self.unitlike(at, ty);
        }
    }

    fn solve_type_numerics(&mut self) {
        let constraints: Vec<_> = self.type_numeric.drain(..).collect();
        for (at, ty) in constraints {
            match self.numeric(at, ty) {
                NumericResult::Ok => {}
                NumericResult::Unsolved(at, ty) => {
                    self.equate(at, ty, Type::Number);
                }

                NumericResult::Error(messages) => self.messages.extend(messages),
            }
        }
    }

    /// Create a unique type.
    fn fresh(&mut self, span: Span) -> Type {
        let counter = self.counts.entry(span).or_insert(0);
        let count = *counter;
        *counter += 1;

        Type::Var(UnifyVar { span, count })
    }

    /// Report an unsolvable constraint.
    fn report_unsolvable(&mut self, constraint: Constraint) {
        let span = match constraint {
            Constraint::Alias { at, .. } => at,
            Constraint::Assignable { at, .. } => at,
            Constraint::Equal(at, _, _) => at,
            Constraint::Field { at, .. } => at,
            Constraint::Instantiated(at, _, _) => at,
            Constraint::InstantiatedAlias(at, _, _) => at,
            Constraint::UnitLike(at, _) => at,
            Constraint::Numeric(at, _) => at,
            Constraint::Textual(at, _) => at,
            Constraint::TypeNumeric(at, _) => at,
        };

        self.at(span).ambiguous();
    }

    fn at(&mut self, span: Span) -> MessageMaker<&mut Vec<Message>> {
        MessageMaker::new(&mut self.messages, span)
    }

    fn common_db(&self) -> &'db dyn zippy_common::Db {
        <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(self.db)
    }
}

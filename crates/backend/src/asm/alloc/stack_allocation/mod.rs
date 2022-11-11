mod apply;
mod fit;
mod procedure;
mod unavailable;

use std::collections::HashMap;
use std::marker::PhantomData;

use common::lir::{Program, Type};
use common::message::Messages;
use common::names::{Name, Names};
use common::Driver;

use self::apply::apply;
use crate::asm::{AllocConstraints, Place, ProcedureAllocation};

pub fn allocate<Constraints: AllocConstraints>(
    driver: &mut impl Driver,
    names: &Names,
    program: Program,
) -> Program {
    let mut allocator: Allocator<Constraints> = Allocator::new(names, program);
    allocator.collect_conventions();
    let (result, messages) = allocator.allocate();

    driver.report(messages);

    result
}

struct Allocation {
    /// Maps virtual registers to a frame offset-size pair.
    ///
    /// Negative offsets refer to frame offset from the previous frame. The assumption is that the frame starts at frame
    /// offset `0`.
    pub mapping: HashMap<usize, (Place, usize)>,

    /// The total amount of frame space allocated for this procedure.
    pub frame_space: usize,
}

struct Allocator<'a, Constraints> {
    procedures: HashMap<Name, ProcedureAllocation>,
    messages: Messages,

    mapping: HashMap<usize, (Place, usize)>,
    frame_space: usize,

    program: Program,
    names: &'a Names,
    _constraints: PhantomData<Constraints>,
}

impl<'a, Constraints: AllocConstraints> Allocator<'a, Constraints> {
    pub fn new(names: &'a Names, program: Program) -> Self {
        Self {
            procedures: HashMap::new(),
            messages: Messages::new(),

            mapping: HashMap::new(),
            frame_space: 0,

            program,
            names,
            _constraints: PhantomData,
        }
    }

    pub fn collect_conventions(&mut self) {
        for name in self.program.procs.keys() {
            let convention = self.program.info.get_convention(name).unwrap();
            let ty = self.program.context.get(name);
            let (args, rets) = match self.program.types.get(&ty) {
                Type::Fun(args, rets) => (args, rets),
                _ => unreachable!(),
            };

            let convention =
                match Constraints::convention(&self.program.types, convention, args, rets) {
                    Some(convention) => convention,
                    None => {
                        let span = self.names.get_span(name);
                        self.messages.at(span).compile_unsupported_convention(
                            Constraints::NAME,
                            convention.to_string(),
                        );

                        ProcedureAllocation {
                            arguments: args.iter().map(|_| Place::Argument(0)).collect(),
                            returns: rets.iter().map(|_| Place::Argument(0)).collect(),
                        }
                    }
                };

            self.procedures.insert(*name, convention);
        }
    }

    pub fn allocate(mut self) -> (Program, Messages) {
        let mut procs = HashMap::with_capacity(self.program.procs.len());

        for (name, procedure) in self.program.procs.drain().collect::<Vec<_>>() {
            let allocation = self.allocate_procedure(&name, &procedure);
            let procedure = apply(allocation, procedure);
            procs.insert(name, procedure);
        }

        let program = Program {
            procs,
            values: self.program.values,
            types: self.program.types,
            context: self.program.context,
            info: self.program.info,
        };

        (program, self.messages)
    }

    fn convention(&self, name: &Name) -> &ProcedureAllocation {
        self.procedures.get(name).unwrap()
    }

    fn map(&mut self, register: usize, offset: Place, size: usize) {
        assert!(self.mapping.insert(register, (offset, size)).is_none());
    }
}

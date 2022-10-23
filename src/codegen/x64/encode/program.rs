use super::super::repr::Program;
use super::Encoder;

impl Encoder {
    pub fn encode_program(&mut self, program: Program) {
        for (name, procedure) in program.procedures {
            self.encode_procedure(name, procedure);
        }
    }
}

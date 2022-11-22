use super::{Error, Interpreter};

impl Interpreter {
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.step() {
                Ok(()) => {}
                Err(Error::NothingLeft) => {
                    self.merge_down();

                    if self.worklist.is_empty() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Error> {
        self.single_step()
    }
}

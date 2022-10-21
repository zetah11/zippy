use std::env;
use std::io::{self, Write};

use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config, DisplayStyle};
use console::{style, Term};

use corollary::message::Messages;
use corollary::{Driver, EvalAmount};

pub struct ConsoleDriver {
    files: SimpleFiles<String, String>,
    writer: StandardStream,
    term: Term,
    config: Config,

    preserve_output: bool,
    partial_eval: EvalAmount,
}

impl ConsoleDriver {
    pub fn new(files: SimpleFiles<String, String>) -> Self {
        Self {
            files,
            writer: StandardStream::stderr(ColorChoice::Auto),
            term: Term::stderr(),
            config: Config {
                display_style: DisplayStyle::Rich,
                ..Default::default()
            },

            preserve_output: env::var("COR_PRESERVE_OUTPUT").is_ok(),
            partial_eval: if env::var("COR_NO_EVAL").is_ok() {
                EvalAmount::None
            } else {
                EvalAmount::Full
            },
        }
    }

    fn clear_line(&mut self) -> io::Result<()> {
        if !self.preserve_output {
            self.term.clear_line()
        } else {
            Ok(())
        }
    }
}

impl Driver for ConsoleDriver {
    fn report(&mut self, messages: Messages) {
        for msg in messages.msgs {
            term::emit(&mut self.writer, &self.config, &self.files, &msg).unwrap();
        }
    }

    fn report_eval(&mut self, at: String) {
        self.clear_line().unwrap();
        write!(self.term, "{}: evaluating '{at}'", style("note").green()).unwrap();

        if self.preserve_output {
            writeln!(self.term).unwrap();
        }
    }

    fn done_eval(&mut self) {
        self.clear_line().unwrap();
    }

    fn entry_name(&mut self) -> Option<String> {
        Some("main".into())
    }

    fn eval_amount(&mut self) -> EvalAmount {
        self.partial_eval
    }
}

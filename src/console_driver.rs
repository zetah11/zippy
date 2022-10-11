use std::env;
use std::io::{self, Write};

use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config, DisplayStyle};
use console::{style, Term};

use corollary::message::Messages;
use corollary::Driver;

pub struct ConsoleDriver {
    files: SimpleFiles<String, String>,
    writer: StandardStream,
    term: Term,
    config: Config,
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
        }
    }

    fn clear_line(&mut self) -> io::Result<()> {
        if env::var("PRESERVE_OUTPUT").is_ok() {
            self.term.write_line("")
        } else {
            self.term.clear_line()
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
    }

    fn done_eval(&mut self) {
        self.clear_line().unwrap();
    }
}

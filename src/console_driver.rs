use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config, DisplayStyle};

use corollary::message::Messages;
use corollary::Driver;

pub struct ConsoleDriver {
    files: SimpleFiles<String, String>,
    writer: StandardStream,
    config: Config,
}

impl ConsoleDriver {
    pub fn new(files: SimpleFiles<String, String>) -> Self {
        Self {
            files,
            writer: StandardStream::stderr(ColorChoice::Auto),
            config: Config {
                display_style: DisplayStyle::Rich,
                ..Default::default()
            },
        }
    }
}

impl Driver for ConsoleDriver {
    fn report(&mut self, messages: Messages) {
        for msg in messages.msgs {
            term::emit(&mut self.writer, &self.config, &self.files, &msg).unwrap();
        }
    }
}

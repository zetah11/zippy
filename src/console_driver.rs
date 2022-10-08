use std::io::{self, Write};

use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{
    Color, ColorChoice, ColorSpec, StandardStream, WriteColor,
};
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

    fn report_eval(&mut self, at: String) {
        write_eval(&mut self.writer, at).unwrap();
    }
}

fn write_eval(stream: &mut StandardStream, at: String) -> io::Result<()> {
    stream.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    write!(stream, "note")?;

    stream.reset()?;
    write!(stream, ": evaluating '{at}'\r")?;

    Ok(())
}

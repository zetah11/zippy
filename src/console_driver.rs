use std::env;
use std::fs::{DirBuilder, File};
use std::io::{self, Write};
use std::path::PathBuf;

use codespan_reporting::diagnostic as cr;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config, DisplayStyle};
use console::{style, Term};

use zippy_common::message::{self, Messages};
use zippy_common::{Driver, EvalAmount, IrOutput};

use super::args::Arguments;

pub struct ConsoleDriver {
    files: SimpleFiles<String, String>,
    writer: StandardStream,
    term: Term,
    config: Config,

    preserve_output: bool,
    ir_output: bool,
    partial_eval: EvalAmount,

    artifacts: PathBuf,
}

impl ConsoleDriver {
    pub fn new(args: &Arguments, files: SimpleFiles<String, String>) -> Self {
        let opts = args.options();
        Self {
            files,
            writer: StandardStream::stderr(ColorChoice::Auto),
            term: Term::stderr(),
            config: Config {
                display_style: DisplayStyle::Rich,
                ..Default::default()
            },

            preserve_output: env::var("COR_PRESERVE_OUTPUT").is_ok() || opts.preserve_output,
            ir_output: env::var("COR_OUTPUT_IR").is_ok() || opts.output_ir,
            partial_eval: if env::var("COR_NO_EVAL").is_ok() || opts.no_eval {
                EvalAmount::None
            } else {
                EvalAmount::Full
            },

            artifacts: args.options().artifacts.clone(),
        }
    }

    fn clear_line(&mut self) -> io::Result<()> {
        if !self.preserve_output {
            self.term.clear_line()
        } else {
            Ok(())
        }
    }

    fn write_ir_file(&self, at: IrOutput, data: String) -> anyhow::Result<()> {
        DirBuilder::new()
            .recursive(true)
            .create(self.artifacts.as_path())?;

        let name = match at {
            IrOutput::Mir(name) => self.artifacts.join(format!("{name}.mir.z")),
        };

        let mut file = File::create(name)?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }
}

impl Driver for ConsoleDriver {
    fn report(&mut self, messages: Messages) {
        for msg in messages.msgs {
            let severity = match msg.severity {
                message::Severity::Bug => cr::Severity::Bug,
                message::Severity::Error => cr::Severity::Error,
                message::Severity::Warning => cr::Severity::Warning,
                message::Severity::Note => cr::Severity::Note,
                message::Severity::Help => cr::Severity::Help,
            };

            let labels = msg
                .labels
                .into_iter()
                .map(|label| {
                    let style = match label.style {
                        message::LabelStyle::Primary => cr::LabelStyle::Primary,
                        message::LabelStyle::Secondary => cr::LabelStyle::Secondary,
                    };

                    cr::Label::new(style, label.span.file, label.span).with_message(label.message)
                })
                .collect();

            let msg = cr::Diagnostic {
                severity,
                code: msg.code,
                message: msg.message,
                labels,
                notes: msg.notes,
            };

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

    fn output_ir(&mut self, at: IrOutput, data: impl FnOnce() -> String) {
        if self.ir_output {
            if let Err(e) = self.write_ir_file(at, data()) {
                eprintln!("error writing ir file: {e}");
            }
        }
    }

    fn entry_name(&mut self) -> Option<String> {
        Some("main".into())
    }

    fn eval_amount(&mut self) -> EvalAmount {
        self.partial_eval
    }
}

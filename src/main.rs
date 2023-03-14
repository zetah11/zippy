mod cli;
mod database;
mod lsp;
mod meta;
mod output;
mod pretty;
mod project;
mod span;

fn main() {
    let mut args = std::env::args();
    let program_name = match args.next() {
        Some(arg) => arg,
        None => print_usage_info("zc"),
    };

    let result = match args.next().as_ref().map(AsRef::as_ref) {
        Some("check") => {
            let args: Vec<_> = args.collect();
            cli::check(args.contains(&String::from("--dot")))
        }
        Some("lsp") => lsp::lsp(),
        _ => print_usage_info(&program_name),
    };

    result.unwrap()
}

fn print_usage_info(program_name: &str) -> ! {
    eprintln!(
        "{} version {}-{}",
        meta::COMPILER_NAME,
        meta::VERSION,
        meta::TAG
    );
    eprintln!("usage: {} <command>", program_name);
    eprintln!();
    eprintln!("available commands:");
    eprintln!("  check    check the project for errors");
    eprintln!(
        "  lsp      run {} as a language server on stdio",
        meta::COMPILER_NAME
    );

    // eugh whatever
    std::process::exit(1)
}

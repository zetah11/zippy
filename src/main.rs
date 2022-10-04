mod console_driver;

use codespan_reporting::files::SimpleFiles;
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::parse::parse;
use corollary::resolve::resolve;
use corollary::tyck::typeck;

use console_driver::ConsoleDriver;

fn main() {
    let src = r#"
        let f : 0 upto 10 -> 0 upto 10 = x => x
        let x : 0 upto 10 = f 5
    "#;
    let mut files = SimpleFiles::new();
    let file = files.add("main.z".into(), src.into());

    let mut driver = ConsoleDriver::new(files);

    let toks = lex(&mut driver, src, file);
    let expr = parse(&mut driver, toks, file);
    let (expr, names) = resolve(&mut driver, expr);
    let expr = typeck(&mut driver, expr);
    let (types, expr) = elaborate(&mut driver, expr);

    println!("{expr:?}");
    println!("{names:?}");
    println!("{types:?}");
}

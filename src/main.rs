mod console_driver;

use codespan_reporting::files::SimpleFiles;
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::mir::pretty::Prettier;
use corollary::parse::parse;
use corollary::resolve::resolve;
use corollary::tyck::typeck;

use console_driver::ConsoleDriver;

fn main() {
    env_logger::init();

    let src = r#"
        let f: (0 upto 10 -> 0 upto 10) -> 0 upto 10 -> 0 upto 10 = f => x => f (f (f (f x)))
        let g: 0 upto 10 -> 0 upto 10 = x => x
        let h: 0 upto 10 = f g 5
        let i: 0 upto 10 = f (f g) 7
    "#;
    let mut files = SimpleFiles::new();
    let file = files.add("main.z".into(), src.into());

    let mut driver = ConsoleDriver::new(files);

    let toks = lex(&mut driver, src, file);
    let expr = parse(&mut driver, toks, file);
    let (expr, mut names) = resolve(&mut driver, expr);
    let expr = typeck(&mut driver, expr);
    let (_types, decls) = elaborate(&mut driver, &mut names, expr);

    let prettier = Prettier::new(&names);

    for def in decls.values.iter() {
        let pat = prettier.pretty_pat(&def.pat);
        let exp = prettier.pretty_expr(&def.bind);
        println!("let {pat} = {exp}");
    }
}

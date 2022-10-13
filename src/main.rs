mod console_driver;

use codespan_reporting::files::SimpleFiles;
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::mir::pretty::Prettier;
use corollary::parse::parse;
use corollary::resolve::{resolve, ResolveRes};
use corollary::tyck::typeck;

use console_driver::ConsoleDriver;

fn main() {
    env_logger::init();

    let src = r#"
        let main: 0 upto 1 -> 0 upto 10 =
            ? => (f (f (f (f id)))) 5
        
        let id = f (x => x)

        let f: (0 upto 10 -> 0 upto 10) -> (0 upto 10 -> 0 upto 10) =
            g => g
    "#;
    let mut files = SimpleFiles::new();
    let file = files.add("main.z".into(), src.into());

    let mut driver = ConsoleDriver::new(files);

    let toks = lex(&mut driver, src, file);
    let decls = parse(&mut driver, toks, file);
    let ResolveRes {
        decls,
        mut names,
        entry,
    } = resolve(&mut driver, decls);

    let tyckres = typeck(&mut driver, decls);
    let (types, context, decls) = elaborate(&mut driver, &mut names, tyckres, entry);

    let prettier = Prettier::new(&names, &types).with_width(20);

    for (name, ty) in context.iter() {
        println!(
            "{}: {}",
            prettier.pretty_name(name),
            prettier.pretty_type(ty)
        );
    }

    println!();

    println!("{}", prettier.pretty_decls(&decls));
}

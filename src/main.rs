mod console_driver;

use codespan_reporting::files::SimpleFiles;
use corollary::asm::asm;
use corollary::codegen::x64::{self, codegen, CONSTRAINTS};
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::parse::parse;
use corollary::resolve::{resolve, ResolveRes};
use corollary::tyck::typeck;

use console_driver::ConsoleDriver;

fn main() {
    env_logger::init();

    let src = r#"
        let main: 0 upto 1 -> ? =
          ? => apply (id, 5)
        
        let id = x => apply ((y => y), x)

        let apply: (10 -> 10) * 10 -> 10 =
          (f, x) => f x
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

    let program = asm(CONSTRAINTS, &types, &context, entry, decls);

    let code = codegen(&mut names, entry, program);
    println!("{}", x64::pretty_program(&names, &code));

    let code = x64::encode(code);
    for byte in code {
        print!("{byte:>2x} ");
    }

    println!();
}

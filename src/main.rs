mod console_driver;

use codespan_reporting::files::SimpleFiles;
use corollary::asm::{asm, Constraints, RegisterInfo};
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
        let main: 0 upto 1 -> ? =
          ? => apply (id, 5)

        let id = x => apply ((y => y), x)

        let apply: (0 upto 10 -> 0 upto 10) * (0 upto 10) -> 0 upto 10 =
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
    println!();

    let program = asm(X64_CONSTRAITNS, &types, &context, entry, decls);
    println!("{program:?}");
}

const X64_CONSTRAITNS: Constraints = Constraints {
    #[rustfmt::skip]
    registers: &[
        RegisterInfo { size: 8, name: "rdi" },
        RegisterInfo { size: 8, name: "rsi" },
        RegisterInfo { size: 8, name: "rdx" },
        RegisterInfo { size: 8, name: "rcx" },
        RegisterInfo { size: 8, name: "r8" },
        RegisterInfo { size: 8, name: "r9" },
        RegisterInfo { size: 8, name: "rbx" },
        RegisterInfo { size: 8, name: "r10" },
        RegisterInfo { size: 8, name: "r11" },
        RegisterInfo { size: 8, name: "r12" },
        RegisterInfo { size: 8, name: "r13" },
        RegisterInfo { size: 8, name: "r14" },
        RegisterInfo { size: 8, name: "r15" },
    ],
};

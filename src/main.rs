mod console_driver;

use codespan_reporting::files::SimpleFiles;
use corollary::elab::elaborate;
use corollary::lex::lex;
use corollary::mir::pretty::Prettier;
use corollary::parse::parse;
use corollary::resolve::{resolve, ResolveRes};
use corollary::tyck::typeck;

use corollary::asm::alloc::{regalloc, Constraints};
use corollary::lir::{Block, Branch, Cond, Inst, ProcBuilder, Register, Target, Value};

use console_driver::ConsoleDriver;

fn main() {
    env_logger::init();

    let src = r#"
        let main: 1 -> ? =
            ? => f (f (f id)) value

        let value: 10 = 5

        let id = f (x => x)

        let f: (10 -> 10) -> (10 -> 10) =
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

    // ------------------
    /*
    start:
        a = 1
        b = 2
        jmp ifso if a < b else otherwise

    ifso:
        push a
        pop c
        jmp end

    otherwise:
        c = 3
        jmp end

    end:
        return c
    */

    let mut builder = ProcBuilder::new();

    let a = Register::Virtual(0);
    let b = Register::Virtual(1);
    let c = Register::Virtual(2);

    let exit = builder.add(Block {
        insts: vec![],
        branch: Branch::Return(Value::Register(c)),
    });

    let consequence = builder.add(Block {
        insts: vec![
            Inst::Push(Value::Register(a)),
            Inst::Pop(Target::Register(c)),
        ],
        branch: Branch::Jump(exit),
    });

    let alternative = builder.add(Block {
        insts: vec![Inst::Move(Target::Register(c), Value::Register(b))],
        branch: Branch::Jump(exit),
    });

    let entry = builder.add(Block {
        insts: vec![
            Inst::Move(Target::Register(a), Value::Integer(1)),
            Inst::Move(Target::Register(b), Value::Integer(2)),
        ],
        branch: Branch::JumpIf {
            conditional: Cond::Less,
            left: Value::Register(a),
            right: Value::Register(b),
            consequence,
            alternative,
        },
    });

    let proc = builder.build(entry, exit);
    let allocd = regalloc(Constraints { max_physical: 16 }, proc);
    println!("{allocd:?}");
}

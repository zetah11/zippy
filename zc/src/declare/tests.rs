use std::sync::Arc;

use super::Declare;
use crate::inputs::Inputs;
use crate::name::{BareData, NameInterner};
use crate::parse::hir::ExprNode;
use crate::source::SourceId;
use crate::ZcDatabase;

#[test]
fn declare_names() {
    let src = r#"
        fun main(): Int
            return x
        where
            let x: Int = 10
        end
        
        type Int = 0 upto 100
    "#;

    let mut db = ZcDatabase::default();
    let id = SourceId::new();
    db.set_input_file(id, Arc::new(src.into()));

    let (tree, scope) = db.decl(id);

    let top_scope = &tree.scope;

    let top_scope = scope.get(top_scope);
    assert_eq!(top_scope.names.len(), 2);
    for name in ["main", "Int"] {
        let name = db.intern_bare(BareData(name.into()));
        assert!(top_scope.names.iter().any(|(n, _)| *n == name));
    }

    let main = db.intern_bare(BareData("main".into()));
    let main = match tree.values.get(&main) {
        Some(main) => main,
        None => panic!("no top-level 'main'"),
    };

    let body = match &main.body.node {
        ExprNode::Fun(_, block, _) => block,
        _ => panic!("'main' is not a fun"),
    };

    let main_scope = scope.get(&body.decls.scope);
    let x = db.intern_bare(BareData("x".into()));
    assert_eq!(main_scope.names.len(), 1);
    assert_eq!(main_scope.names[0].0, x);
}

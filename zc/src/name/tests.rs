use super::{NameStore, QualifiedName};

/// Check that the store handles cascades with removes correctly.
#[test]
fn name_add_remove() {
    let mut store = NameStore::new();

    let root = store.add_qualified(QualifiedName::Root("test".into()));
    let int = store.add_qualified(QualifiedName::Contained {
        container: root,
        name: "int".into(),
    });
    let main = store.add_qualified(QualifiedName::Contained {
        container: root,
        name: "main".into(),
    });
    let x = store.add_qualified(QualifiedName::Contained {
        container: main,
        name: "x".into(),
    });
    let f = store.add_qualified(QualifiedName::Contained {
        container: main,
        name: "f".into(),
    });
    let a = store.add_qualified(QualifiedName::Contained {
        container: f,
        name: "a".into(),
    });
    let help = store.add_qualified(QualifiedName::Contained {
        container: root,
        name: "help".into(),
    });

    store.remove_name(&main);

    for name in [root, int, help] {
        assert!(store.has_name(&name));
    }

    for name in [main, x, f, a] {
        assert!(!store.has_name(&name));
    }
}

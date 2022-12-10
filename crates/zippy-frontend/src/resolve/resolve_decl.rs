use zippy_common::hir::{Decls, TypeDef, ValueDef};
use zippy_common::message::Span;
use zippy_common::names::{Actual, Path};

use super::{Name, Resolver};

impl Resolver {
    pub fn resolve_decls(&mut self, decls: Decls) -> Decls<Name> {
        let mut values = Vec::with_capacity(decls.values.len());
        let mut types = Vec::with_capacity(decls.types.len());

        for def in decls.values {
            values.push(self.resolve_value_def(def));
        }

        for def in decls.types {
            types.push(self.resolve_type_def(def));
        }

        Decls { values, types }
    }

    fn resolve_value_def(&mut self, def: ValueDef) -> ValueDef<Name> {
        let pat = self.resolve_pat(def.pat);

        self.enter(def.span, def.id);
        let implicits = def
            .implicits
            .into_iter()
            .map(|ty| self.resolve_type_name(ty))
            .collect();
        let anno = self.resolve_type(def.anno);
        let bind = self.resolve_expr(def.bind);

        self.exit();

        ValueDef {
            span: def.span,
            id: def.id,
            implicits,
            pat,
            anno,
            bind,
        }
    }

    fn resolve_type_def(&mut self, def: TypeDef) -> TypeDef<Name> {
        let pat = self.resolve_pat(def.pat);

        self.enter(def.span, def.id);

        let anno = self.resolve_type(def.anno);
        let bind = self.resolve_type(def.bind);

        self.exit();

        TypeDef {
            span: def.span,
            id: def.id,
            pat,
            anno,
            bind,
        }
    }

    fn resolve_type_name(&self, ty: (String, Span)) -> (Name, Span) {
        let path = Path::new(self.context(), Actual::Lit(ty.0));
        // should never fail
        let name = self.names.lookup(&path).unwrap();
        (name, ty.1)
    }
}

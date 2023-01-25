use super::path::NamePart;
use super::Resolver;
use crate::resolved::{Decls, TypeDef, ValueDef};
use crate::unresolved;

impl Resolver<'_> {
    pub fn resolve_decls(&mut self, decls: unresolved::Decls) -> Decls {
        let unresolved_values = decls.values(self.db);
        let unresolved_types = decls.types(self.db);

        let mut values = Vec::with_capacity(unresolved_values.len());
        let mut types = Vec::with_capacity(unresolved_values.len());

        for def in unresolved_values.iter().cloned() {
            let value = self.resolve_value_def(&mut values, def);
            values.push(value);
        }

        for def in unresolved_types.iter().cloned() {
            types.push(self.resolve_type_def(&mut values, def));
        }

        Decls::new(self.db, values, types)
    }

    fn resolve_value_def(
        &mut self,
        values: &mut Vec<ValueDef>,
        def: unresolved::ValueDef,
    ) -> ValueDef {
        let pat = self.resolve_pat(values, def.pat);

        self.in_scope(NamePart::Scope(def.id), |this| {
            let implicits = def
                .implicits
                .into_iter()
                .map(|(name, span)| this.lookup(span, name).unwrap())
                .collect();

            let anno = this.resolve_type(values, def.anno);
            let bind = this.resolve_expr(values, def.bind);

            ValueDef {
                span: def.span,
                pat,
                implicits,
                anno,
                bind,
            }
        })
    }

    fn resolve_type_def(
        &mut self,
        values: &mut Vec<ValueDef>,
        def: unresolved::TypeDef,
    ) -> TypeDef {
        let pat = self.resolve_pat(values, def.pat);

        self.in_scope(NamePart::Scope(def.id), |this| {
            let anno = this.resolve_type(values, def.anno);
            let bind = this.resolve_type(values, def.bind);

            TypeDef {
                span: def.span,
                pat,
                anno,
                bind,
            }
        })
    }
}

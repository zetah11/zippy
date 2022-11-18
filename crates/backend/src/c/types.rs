use common::mir::{Type, TypeId};

use super::Emitter;

macro_rules! within {
    ($v:expr, $w:expr, $t:ty) => {
        $v >= <$t>::MIN as i64
            && $v <= <$t>::MAX as i64
            && $w >= <$t>::MIN as i64
            && $w <= <$t>::MAX as i64
    };
}

macro_rules! range_to_type {
    ( $lo:expr, $hi:expr, $($t:ty => $value:tt),*, else => $fallback:expr ) => {
        match () {
            $( _ if within!($lo, $hi, $t) => $value ),*,
            _ => $fallback,
        }
    };
}

impl Emitter<'_> {
    pub fn typename(&mut self, ty: &TypeId) -> &str {
        if !self.type_map.contains_key(ty) {
            let name = self.make_typename(ty);
            self.type_map.insert(*ty, name);
        }

        self.type_map.get(ty).unwrap()
    }

    fn make_typename(&mut self, ty: &TypeId) -> String {
        match self.types.get(ty) {
            Type::Range(lo, hi) => {
                let ty = range_to_type! {
                    *lo, *hi,
                    i8 => "signed char",
                    u8 => "unsigned char",
                    i16 => "signed short",
                    u16 => "unsigned short",
                    i32 => "int",
                    u32 => "unsigned",
                    i64 => "long long",
                    else => unreachable!()
                };

                ty.into()
            }

            Type::Product(ties) => {
                let ty = self.make_struct(ties);
                let name = self.fresh_typename();

                self.typedef(&name, &ty, "");
                name
            }

            Type::Fun(args, rets) => {
                let ret = match &rets[..] {
                    [] => "void".into(),
                    [ret] => self.typename(ret).into(),
                    _ => self.make_struct(rets),
                };

                let args: Vec<_> = args
                    .iter()
                    .map(|ty| self.typename(ty).to_string())
                    .collect();
                let args = args.join(", ");

                let pre = format!("{ret} (*");
                let post = format!(")({args})");
                let name = self.fresh_typename();
                self.typedef(&name, &pre, &post);
                name
            }

            Type::Invalid => {
                let name = self.fresh_typename();
                self.typedef(&name, "void *", "");
                name
            }
        }
    }

    pub fn make_struct(&mut self, of: &[TypeId]) -> String {
        let ties: Vec<_> = of
            .iter()
            .enumerate()
            .map(|(ndx, ty)| format!("{} f{ndx};", self.typename(ty)))
            .collect();

        format!("struct {{\n\t{}\n}}", ties.join("\n\t"))
    }

    fn typedef(&mut self, name: &str, pre: &str, post: &str) {
        self.typedefs
            .push_str(&format!("typedef {pre} {name}{post};\n"));
    }

    fn fresh_typename(&mut self) -> String {
        let counter = self.type_name;
        self.type_name += 1;
        format!("t{counter}")
    }
}

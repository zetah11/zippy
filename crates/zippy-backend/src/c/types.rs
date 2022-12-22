use zippy_common::message::{Messages, Span};
use zippy_common::mir::{StaticValueNode, Type, TypeId};
use zippy_common::names::Name;
use zippy_common::Number;

use super::Emitter;

macro_rules! within {
    ($v:expr, $w:expr, $t:ty) => {
        $v >= <$t>::MIN && $v <= <$t>::MAX && $w >= <$t>::MIN && $w <= <$t>::MAX
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

    pub fn make_struct(&mut self, of: &[TypeId]) -> String {
        let ties: Vec<_> = of
            .iter()
            .enumerate()
            .map(|(ndx, ty)| format!("{} f{ndx};", self.typename(ty)))
            .collect();

        format!("struct {{\n\t{}\n}}", ties.join("\n\t"))
    }

    fn make_typename(&mut self, ty: &TypeId) -> String {
        match self.types.get(ty) {
            Type::Range(lo, hi) => self.make_integer_type(*lo, *hi),

            Type::Product(ties) => {
                let ty = self.make_struct(&ties.clone());
                let name = self.fresh_typename();

                self.typedef(&name, &ty, "");
                name
            }

            Type::Fun(args, rets) => {
                let args = args.clone();
                let rets = rets.clone();
                let ret = match &rets[..] {
                    [] => "void".into(),
                    [ret] => self.typename(ret).into(),
                    _ => self.make_struct(&rets),
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

            Type::Number => {
                unreachable!("values of type <number> should never be reachable from user code")
            }

            Type::Invalid => {
                let name = self.fresh_typename();
                self.typedef(&name, "void *", "");
                name
            }
        }
    }

    fn make_integer_type(&mut self, lo: Name, hi: Name) -> String {
        let lo_span = self.names.get_span(&lo);
        let hi_span = self.names.get_span(&hi);

        let mut messages = Messages::new();

        let (lo, _) = self.get_bounds(&mut messages, lo_span, &lo);
        let (_, hi) = self.get_bounds(&mut messages, hi_span, &hi);

        let ty = range_to_type! {
            lo.clone(), hi.clone(),
            u8 => "unsigned char",
            i8 => "signed char",
            u16 => "unsigned short",
            i16 => "signed short",
            u32 => "unsigned",
            i32 => "int",
            i64 => "long long",
            else => unreachable!()
        };

        self.messages.merge(messages);

        ty.into()
    }

    fn get_bounds(&self, messages: &mut Messages, at: Span, name: &Name) -> (&Number, &Number) {
        let value = self.values.get(name).unwrap();

        match &value.node {
            StaticValueNode::Num(value) => (value, value),
            StaticValueNode::LateInit(block) => {
                let ty = self.types.get(&block.ty);
                match ty {
                    Type::Range(lo, hi) => {
                        let (lo, _) = self.get_bounds(messages, value.span, lo);
                        let (_, hi) = self.get_bounds(messages, value.span, hi);
                        (lo, hi)
                    }
                    Type::Invalid => todo!(),
                    Type::Fun(..) | Type::Product(..) => unreachable!(),

                    Type::Number => {
                        messages.at(at).compile_unconstrained_range();
                        todo!()
                    }
                }
            }
        }
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

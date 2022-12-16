use zippy_common::Number;

pub fn parse_dec(text: &str) -> Number {
    let mut res = 0.into();

    for c in text.chars() {
        match c {
            '0'..='9' => {
                res *= <i32 as Into<Number>>::into(10);
                res += <u32 as Into<Number>>::into(c.to_digit(10).unwrap());
            }
            '_' | '\'' => continue,
            _ => unreachable!(),
        }
    }

    res
}

use zippy_common::Number;

pub fn parse_dec(text: &str) -> Number {
    let mut res = Number::from_integer(0.into());

    for c in text.chars() {
        match c {
            '0'..='9' => {
                res *= Number::from_integer(10.into());
                res += Number::from_integer(c.to_digit(10).unwrap().into());
            }
            '_' | '\'' => continue,
            _ => unreachable!(),
        }
    }

    res
}

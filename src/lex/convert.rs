pub fn parse_dec(text: &str) -> u64 {
    let mut res = 0;

    for c in text.chars() {
        match c {
            '0'..='9' => {
                res *= 10;
                res += c.to_digit(10).unwrap() as u64;
            }
            '_' | '\'' => continue,
            _ => unreachable!(),
        }
    }

    res
}

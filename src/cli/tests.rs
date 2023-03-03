use super::format;
use zippy_common::text;

#[test]
fn format_plain_works() {
    let formatted = format::plain(text![
        "hello ",
        (code "world!")
    ]);

    assert_eq!("hello `world!`", &formatted);
}

#[test]
fn format_indented_small_is_unchanged() {
    let give_up = 0;
    let width = Some(40);
    let indent = 8;
    let formatted = format::indented(
        give_up,
        width,
        indent,
        text![
            "hello ",
            (code "world!")
        ],
    );

    assert_eq!("hello `world!`", &formatted);
}

#[test]
fn format_indented_many_words() {
    let give_up = 0;
    let width = Some(10);
    let indent = 0;
    let formatted = format::indented(
        give_up,
        width,
        indent,
        text!["abc def ghi jkl  mno pqr stu vwx yz."],
    );

    let expected = "abc def\nghi jkl\nmno pqr\nstu vwx\nyz.";

    assert_eq!(expected, &formatted);
}

#[test]
fn format_indented_give_up() {
    let give_up = 10;
    let width = Some(15);
    let indent = 8;
    let formatted = format::indented(
        give_up,
        width,
        indent,
        text!["abc def ghi jkl mno pqr stu vwx yz."],
    );

    let expected = "abc def ghi jkl mno pqr stu vwx yz.";

    assert_eq!(expected, &formatted);
}

#[test]
fn format_indented_works() {
    let give_up = 10;
    let width = Some(20);
    let indent = 4;
    let formatted = format::indented(
        give_up,
        width,
        indent,
        text!["there is text here which should eventually get the proper indentation hopefully pleasepleasepleaseplease"],
    );

    let expected = r#"there is text
    here which
    should
    eventually get
    the proper
    indentation
    hopefully please
    pleasepleaseplea
    se"#;

    assert_eq!(expected, &formatted);
}

#[test]
fn format_cutoff_short() {
    let width = Some(20);
    let highlight = (1, 5);
    let (formatted, start, end) = format::cutoff(width, highlight, "beep boop baap".into());

    let expected = "beep boop baap";

    assert_eq!(expected, &formatted);
    assert_eq!(highlight, (start, end));
}

#[test]
fn format_cutoff_middle() {
    let width = Some(15);
    let highlight = (10, 15);
    let (formatted, start, end) = format::cutoff(
        width,
        highlight,
        "abc def ghi jkl mno pqr stu vwx yz.".into(),
    );

    let expected = "...ghi jkl m...";
    let highlight = (5, 10);

    eprintln!("{formatted:?} {start:?} {end:?}");

    assert_eq!(expected, &formatted);
    assert_eq!(highlight, (start, end));
}

#[test]
fn format_cutoff_long_highlight() {
    let width = Some(10);
    let highlight = (5, 15);
    let (formatted, start, end) = format::cutoff(
        width,
        highlight,
        "abc def ghi jkl mno pqr stu vwx yz.".into(),
    );

    let expected = "...ef g...";
    let highlight = (3, 10);

    assert_eq!(expected, &formatted);
    assert_eq!(highlight, (start, end));
}

#[test]
fn format_cutoff_early_in_long_line() {
    let width = Some(10);
    let highlight = (2, 6);
    let (formatted, start, end) = format::cutoff(
        width,
        highlight,
        "abc def ghi jkl mno pqr stu vwx yz.".into(),
    );

    let expected = "abc def...";

    assert_eq!(expected, &formatted);
    assert_eq!(highlight, (start, end));
}

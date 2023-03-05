use crate::database::Database;
use crate::pretty::Prettier;

use super::format;
use zippy_common::messages::Text;
use zippy_common::text;

fn check_plain(expected: &str, text: Text) {
    let db = Database::new();
    let prettier = Prettier::new(&db);
    let formatted = format::plain(&prettier, text);

    assert_eq!(expected, &formatted);
}

fn check_indented(expected: &str, give_up: usize, width: Option<usize>, indent: usize, text: Text) {
    let db = Database::new();
    let prettier = Prettier::new(&db);
    let formatted = format::indented(&prettier, give_up, width, indent, text);

    assert_eq!(expected, &formatted);
}

#[test]
fn format_plain_works() {
    let text = text![
        "hello ",
        (code "world!")
    ];

    check_plain("hello `world!`", text);
}

#[test]
fn format_indented_small_is_unchanged() {
    let give_up = 0;
    let width = Some(40);
    let indent = 8;
    let text = text![
        "hello ",
        (code "world!")
    ];

    check_indented("hello `world!`", give_up, width, indent, text);
}

#[test]
fn format_indented_many_words() {
    let give_up = 0;
    let width = Some(10);
    let indent = 0;
    let text = text!["abc def ghi jkl  mno pqr stu vwx yz."];

    let expected = "abc def\nghi jkl\nmno pqr\nstu vwx\nyz.";

    check_indented(expected, give_up, width, indent, text);
}

#[test]
fn format_indented_give_up() {
    let give_up = 10;
    let width = Some(15);
    let indent = 8;
    let text = text!["abc def ghi jkl mno pqr stu vwx yz."];

    let expected = "abc def ghi jkl mno pqr stu vwx yz.";

    check_indented(expected, give_up, width, indent, text);
}

#[test]
fn format_indented_works() {
    let give_up = 10;
    let width = Some(20);
    let indent = 4;
    let text = text!["there is text here which should eventually get the proper indentation hopefully pleasepleasepleaseplease"];

    let expected = r#"there is text
    here which
    should
    eventually get
    the proper
    indentation
    hopefully please
    pleasepleaseplea
    se"#;

    check_indented(expected, give_up, width, indent, text);
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

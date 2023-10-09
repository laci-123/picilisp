use pretty_assertions::assert_eq;
use super::*;


#[test]
fn highlight_parens_empty() {
    let s  = "";
    let cursor = 1;
    let style = "color: green;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "");
}

#[test]
fn highlight_parens_no_parens() {
    let s  = "The quick brown fox jumps over the lazy dog.";
    let cursor = 10;
    let style = "background-color: blue;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "The quick brown fox jumps over the lazy dog.");
}

#[test]
fn highlight_parens_only_parens() {
           // 1234
    let s  = "(())";
    let cursor = 1;
    let style = "color: cyan;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "<span style='color: cyan;'>(</span>()<span style='color: cyan;'>)</span>");
}

#[test]
fn highlight_parens_cursor_not_on_paren() {
           // 123456789
    let s  = "(+ 1 (+ 2 3))";
    let cursor = 4;
    let style = "color: yellow;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "(+ 1 (+ 2 3))");
}

#[test]
fn highlight_parens_cursor_on_open_paren() {
           // 123456789
    let s  = "(+ 1 (+ 2 3))";
    let cursor = 1;
    let style = "color: green;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "<span style='color: green;'>(</span>+ 1 (+ 2 3)<span style='color: green;'>)</span>");
}

#[test]
fn highlight_parens_cursor_on_close_paren() {
           // 123456789
    let s  = "(list (1) 2 3)";
    let cursor = 9;
    let style = "color: green;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "(list <span style='color: green;'>(</span>1<span style='color: green;'>)</span> 2 3)");
}

#[test]
fn highlight_parens_cursor_on_unbalanced_right() {
           // 123456789
    let s  = "(list (1) 2 3";
    let cursor = 1;
    let style = "color: green;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "<span style='color: red;'>(</span>list (1) 2 3");
}

#[test]
fn highlight_parens_cursor_on_unbalanced_left() {
           // 123456789
    let s  = "list 1 2)";
    let cursor = 9;
    let style = "color: green;";
    let unbalanced_style = "color: red;";
    let hs = highlight_parens(s, cursor, style, unbalanced_style);

    assert_eq!(hs, "list 1 2<span style='color: red;'>)</span>");
}

use super::*;



#[test]
fn highlight_parens_empty() {
    let s = "";
    let c = 0;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::None));
}

#[test]
fn highlight_parens_no_parens() {
          // 0123456789
    let s = "one two three (four)";
    let c = 5;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::None));
}

#[test]
fn highlight_parens_unbalenced_open() {
          // 0123456789
    let s = "one(two(three four";
    let c = 7;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::UnbalancedOpen(7)));
}

#[test]
fn highlight_parens_unbalenced_close() {
          // 0123456789
    let s = "one)two three (four)";
    let c = 3;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::UnbalancedClose(3)));
}

#[test]
fn highlight_parens_ok_on_open() {
          // 0123456789
    let s = "one (two) three (four)";
    let c = 4;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::Ok(4, 8)));
}

#[test]
fn highlight_parens_ok_on_close() {
          // 0123456789
    let s = "one (two) three (four)";
    let c = 8;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::Ok(4, 8)));
}

#[test]
fn highlight_parens_ok_nested() {
          // 0123456789
    let s = "(one (two) three four)";
    let c = 5;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::Ok(5, 9)));
}

#[test]
fn highlight_parens_ok_tight() {
          // 0123456789
    let s = "one two () three (four)";
    let c = 9;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::Ok(8, 9)));
}

#[test]
fn highlight_parens_only_parens() {
          // 0123456789
    let s = "(())()";
    let c = 3;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::Ok(0, 3)));
}

#[test]
fn highlight_parens_unbalenced_open_nested() {
          // 0123456789
    let s = "(one (two (three) four)";
    let c = 0;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::UnbalancedOpen(0)));
}


#[test]
fn highlight_parens_unbalenced_close_nested() {
          // 0123456789
    let s = "((one)))";
    let c = 7;
    let hp = highlight_parens(StringWithCursor { string: s, cursor: c });
    assert!(matches!(hp, HighlightedParens::UnbalancedClose(7)));
}

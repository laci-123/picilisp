use pretty_assertions::assert_eq;
use crate::util::{string_to_list, list_to_string, list_to_vec};
use super::*;


#[test]
fn read_empty() {
    let mut mem = Memory::new();

    let r = read(&mut mem, &[GcRef::nil()], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("nothing").get().as_symbol());

    let input = string_to_list(&mut mem, "   ;comment ");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("nothing").get().as_symbol());
}

#[test]
fn read_number() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "-1235");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_number(), -1235);
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_character() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "%a");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_character(), 'a');
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_escaped_character() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r"%\n");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_character(), '\n');
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_symbol() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "+a-symbol");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get().as_symbol(), mem.symbol_for("+a-symbol").get().as_symbol());
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_whitespace() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "   %A   ");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_character(), 'A');
    assert_eq!(list_to_string(rest).unwrap(), "   ");
}


#[test]
fn read_comment() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, " ;this is a comment\n 10");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_number(), 10);
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_empty_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "()");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert!(result.is_nil());
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "( 1 ;something\n  2 3)");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(list_to_vec(result).unwrap().iter().map(|x| *x.get().as_number()).collect::<Vec<i64>>(), vec![1, 2, 3]);
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_singleton_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(blue-whale)");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let vec = list_to_vec(result).unwrap();
    assert_eq!(vec[0].get().as_symbol(), mem.symbol_for("blue-whale").get().as_symbol());
    assert_eq!(vec.len(), 1);
}

#[test]
fn read_nested_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(1 (%a %b)  2)");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let vec = list_to_vec(result).unwrap();
    assert_eq!(*vec[0].get().as_number(), 1);
    let vec2 = list_to_vec(vec[1].clone()).unwrap();
    assert_eq!(*vec2[0].get().as_character(), 'a');
    assert_eq!(*vec2[1].get().as_character(), 'b');
    assert_eq!(*vec[2].get().as_number(), 2);
}

#[test]
fn read_string_in_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#"(1 "ab"  2)"#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let vec = list_to_vec(result).unwrap();
    assert_eq!(*vec[0].get().as_number(), 1);
    let vec2 = list_to_vec(vec[1].clone()).unwrap();
    assert_eq!(vec2[0].get().as_symbol().get_name(), "list");
    assert_eq!(*vec2[1].get().as_character(), 'a');
    assert_eq!(*vec2[2].get().as_character(), 'b');
    assert_eq!(*vec[2].get().as_number(), 2);
}

#[test]
fn read_string() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#" "The sky is blue." "#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let string = list_to_string(result).unwrap();
    assert_eq!(string, "The sky is blue.");
}

#[test]
fn read_escaped_string() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#" "This is not the end: \". This is a newline: \n." "#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let string = list_to_string(result).unwrap();
    assert_eq!(string, "This is not the end: \". This is a newline: \n.");
}

#[test]
fn read_string_with_special_chars() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#" "The sky isn't pink (for now); Elephants are also not pink." "#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let string = list_to_string(result).unwrap();
    assert_eq!(string, "The sky isn't pink (for now); Elephants are also not pink.");
}

#[test]
fn read_with_remainder() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "10(a b c)");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_number(), 10);
    assert_eq!(list_to_string(rest).unwrap(), "(a b c)");
}

#[test]
fn read_with_remainder_2() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(a b c) 10");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(list_to_vec(result).unwrap().len(), 3);
    assert_eq!(list_to_string(rest).unwrap(), " 10");
}

#[test]
fn read_with_remainder_3() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(a b c ) 10");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(list_to_vec(result).unwrap().len(), 3);
    assert_eq!(list_to_string(rest).unwrap(), " 10");
}

#[test]
fn read_incomplete() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(((");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());

    let input = string_to_list(&mut mem, "(%a %b %c ((");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());

    let input = string_to_list(&mut mem, "(a b c (1 2 3) d");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());

    let input = string_to_list(&mut mem, r#" "something "#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());
}

#[test]
fn read_bad_escape_char() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#" "abc \k def" "#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let error_msg = list_to_string(result.get().as_conscell().get_cdr()).unwrap();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("error").get().as_symbol());
    assert_eq!(error_msg, "'k' is not a valid escape character in a string literal");
}

#[test]
fn read_bad_parens() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, ")");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("error").get().as_symbol());
    assert_eq!(list_to_string(result.get().as_conscell().get_cdr()).unwrap(), "too many closing parentheses");
}


#[test]
fn read_location_atom() {
    let mut mem = Memory::new();

                                       //              123
    let input = string_to_list(&mut mem, ";first line\n  abc");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get().as_symbol(), mem.symbol_for("abc").get().as_symbol());
    assert_eq!(result.get_metadata().unwrap().location.file, None);
    assert_eq!(result.get_metadata().unwrap().location.line, 2);
    assert_eq!(result.get_metadata().unwrap().location.column, 3);
}


#[test]
fn read_location_string() {
    let mut mem = Memory::new();

                                       //                            123 4
    let input = string_to_list(&mut mem, ";first line\n;second line\n   \"some text\"");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get_metadata().unwrap().location.file, None);
    assert_eq!(result.get_metadata().unwrap().location.line, 3);
    assert_eq!(result.get_metadata().unwrap().location.column, 4);
}

#[test]
fn read_location_list() {
    let mut mem = Memory::new();

                                      //  123 45 6789
    let input = string_to_list(&mut mem, "(1 \"2\"  three)");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    let elems = list_to_vec(result).unwrap();
    assert_eq!(elems[0].get_metadata().unwrap().location.file, None);
    assert_eq!(elems[0].get_metadata().unwrap().location.line, 1);
    assert_eq!(elems[0].get_metadata().unwrap().location.column, 2);
    assert_eq!(elems[1].get_metadata().unwrap().location.file, None);
    assert_eq!(elems[1].get_metadata().unwrap().location.line, 1);
    assert_eq!(elems[1].get_metadata().unwrap().location.column, 4);
    assert_eq!(elems[2].get_metadata().unwrap().location.file, None);
    assert_eq!(elems[2].get_metadata().unwrap().location.line, 1);
    assert_eq!(elems[2].get_metadata().unwrap().location.column, 9);
}

#[test]
fn read_location_list_2() {
    let mut mem = Memory::new();

                                      //  1234          12345 6 
    let input = string_to_list(&mut mem, " ( 1;comment\n%2   \"three\")");
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    let elems = list_to_vec(result).unwrap();
    assert_eq!(elems[0].get_metadata().unwrap().location.file, None);
    assert_eq!(elems[0].get_metadata().unwrap().location.line, 1);
    assert_eq!(elems[0].get_metadata().unwrap().location.column, 4);
    assert_eq!(elems[1].get_metadata().unwrap().location.file, None);
    assert_eq!(elems[1].get_metadata().unwrap().location.line, 2);
    assert_eq!(elems[1].get_metadata().unwrap().location.column, 1);
    assert_eq!(elems[2].get_metadata().unwrap().location.file, None);
    assert_eq!(elems[2].get_metadata().unwrap().location.line, 2);
    assert_eq!(elems[2].get_metadata().unwrap().location.column, 6);
}

#[test]
fn read_error_location() {
    let mut mem = Memory::new();

                                          //1234567
    let input = string_to_list(&mut mem, r#" "abc\defg" "#);
    let r = read(&mut mem, &[input], GcRef::nil()).unwrap();
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("error").get().as_symbol());
    let error_location = list_to_vec(result.get().as_conscell().get_car()).unwrap();
    let error_msg      = list_to_string(result.get().as_conscell().get_cdr()).unwrap();
    assert_eq!(error_msg, "'d' is not a valid escape character in a string literal");
    assert_eq!(error_location[0].get().as_symbol().get_name(), "stdin");
    assert_eq!(*error_location[1].get().as_number(), 1);
    assert_eq!(*error_location[2].get().as_number(), 7);
}

#[test]
fn read_continue_rest() {
    let mut mem = Memory::new();

                                       // 123456789
    let input = string_to_list(&mut mem, "cat mouse");
    let start_line = mem.allocate_number(1);
    let start_column = mem.allocate_number(1);
    let r = list_to_vec(read(&mut mem, &[input, start_line, start_column], GcRef::nil()).unwrap()).unwrap();
    let status = r[0].clone();
    let result = r[1].clone();
    let rest   = r[2].clone();
    let line   = r[3].clone();
    let column = r[4].clone();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get().as_symbol().get_name(), "cat");
    assert_eq!(list_to_string(rest.clone()).unwrap(), " mouse");
    assert_eq!(*line.get().as_number(), 1);
    assert_eq!(*column.get().as_number(), 4);

    let r = list_to_vec(read(&mut mem, &[rest, line, column], GcRef::nil()).unwrap()).unwrap();
    let status = r[0].clone();
    let result = r[1].clone();
    let rest   = r[2].clone();
    let line   = r[3].clone();
    let column = r[4].clone();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get().as_symbol().get_name(), "mouse");
    assert_eq!(list_to_string(rest).unwrap(), "");
    assert_eq!(*line.get().as_number(), 1);
    assert_eq!(*column.get().as_number(), 10);
}

#[test]
fn read_continue_rest_newline() {
    let mut mem = Memory::new();

                                       // 123456 01234567
    let input = string_to_list(&mut mem, "  lion\n tiger  ");
    let start_line = mem.allocate_number(1);
    let start_column = mem.allocate_number(1);
    let r = list_to_vec(read(&mut mem, &[input, start_line, start_column], GcRef::nil()).unwrap()).unwrap();
    let status = r[0].clone();
    let result = r[1].clone();
    let rest   = r[2].clone();
    let line   = r[3].clone();
    let column = r[4].clone();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get().as_symbol().get_name(), "lion");
    assert_eq!(list_to_string(rest.clone()).unwrap(), " tiger  ");
    assert_eq!(*line.get().as_number(), 2);
    assert_eq!(*column.get().as_number(), 1);

    let r = list_to_vec(read(&mut mem, &[rest, line, column], GcRef::nil()).unwrap()).unwrap();
    let status = r[0].clone();
    let result = r[1].clone();
    let rest   = r[2].clone();
    let line   = r[3].clone();
    let column = r[4].clone();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(result.get().as_symbol().get_name(), "tiger");
    assert_eq!(list_to_string(rest).unwrap(), "  ");
    assert_eq!(*line.get().as_number(), 2);
    assert_eq!(*column.get().as_number(), 7);
}

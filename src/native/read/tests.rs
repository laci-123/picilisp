use pretty_assertions::assert_eq;
use crate::util::{string_to_list, list_to_string, list_to_vec};
use super::*;


#[test]
fn read_empty() {
    let mut mem = Memory::new();

    let r = read(&mut mem, GcRef::nil());
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());
}

#[test]
fn read_number() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "-123.5");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_number(), -123.5);
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_character() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "%a");
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_number(), 10.0);
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_empty_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "()");
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(list_to_vec(result).unwrap().iter().map(|x| *x.get().as_number()).collect::<Vec<f64>>(), vec![1.0, 2.0, 3.0]);
    assert_eq!(list_to_string(rest).unwrap(), "");
}

#[test]
fn read_singleton_list() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(blue-whale)");
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let vec = list_to_vec(result).unwrap();
    assert_eq!(*vec[0].get().as_number(), 1.0);
    let vec2 = list_to_vec(vec[1].clone()).unwrap();
    assert_eq!(*vec2[0].get().as_character(), 'a');
    assert_eq!(*vec2[1].get().as_character(), 'b');
    assert_eq!(*vec[2].get().as_number(), 2.0);
}

#[test]
fn read_string() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#" "The sky is blue." "#);
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
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
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());

    let string = list_to_string(result).unwrap();
    assert_eq!(string, "The sky isn't pink (for now); Elephants are also not pink.");
}

#[test]
fn read_with_remainder() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "1.0(a b c)");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    let rest = r.get().as_conscell().get_cdr().get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("ok").get().as_symbol());
    assert_eq!(*result.get().as_number(), 1.0);
    assert_eq!(list_to_string(rest).unwrap(), "(a b c)");
}

#[test]
fn read_incomplete() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, "(((");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());

    let input = string_to_list(&mut mem, "(%a %b %c ((");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());

    let input = string_to_list(&mut mem, "(a b c (1 2 3) d");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());

    let input = string_to_list(&mut mem, r#" "something "#);
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("incomplete").get().as_symbol());
}

#[test]
fn read_bad_escape_char() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, r#" "abc \k def" "#);
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("error").get().as_symbol());
    assert_eq!(list_to_string(result).unwrap(), "'k' is not a valid escape character in a string literal");
}

#[test]
fn read_bad_parens() {
    let mut mem = Memory::new();

    let input = string_to_list(&mut mem, ")");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("error").get().as_symbol());
    assert_eq!(list_to_string(result).unwrap(), "too many closing parentheses");

    let input = string_to_list(&mut mem, "(a b (1 2 3) c))");
    let r = read(&mut mem, input);
    let status = r.get().as_conscell().get_car();
    let result = r.get().as_conscell().get_cdr().get().as_conscell().get_car();
    assert_eq!(status.get().as_symbol(), mem.symbol_for("error").get().as_symbol());
    assert_eq!(list_to_string(result).unwrap(), "too many closing parentheses");
}

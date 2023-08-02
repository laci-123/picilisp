use pretty_assertions::assert_eq;
use super::*;


#[test]
fn print_print_atom() {
    let mut mem = Memory::new();

    let x = ExternalReference::nil();
    let p = print(&mut mem, x);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "()");

    let x = mem.allocate_number(-12.3);
    let p = print(&mut mem, x);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "-12.3");

    let x = mem.allocate_character('A');
    let p = print(&mut mem, x);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "%A");

    let x = mem.symbol_for("kitten");
    let p = print(&mut mem, x);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "kitten");

    let x = mem.allocate_function(ExternalReference::nil(), FunctionKind::Lambda, vec![]);
    let p = print(&mut mem, x);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "#<function>");

    let car = mem.allocate_number(99.9);
    let cdr = mem.allocate_character('9');
    let x = mem.allocate_cons(car, cdr);
    let p = print(&mut mem, x);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "(cons 99.9 %9)");
}

#[test]
fn print_print_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_number(2.71), ExternalReference::nil(), mem.allocate_character('$'), mem.symbol_for("puppy")];
    let list = vec_to_list(&mut mem, vec);
    let p = print(&mut mem, list);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "(2.71 () %$ puppy)");
}

#[test]
fn print_print_singleton_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.symbol_for("only-this-one")];
    let list = vec_to_list(&mut mem, vec);
    let p = print(&mut mem, list);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "(only-this-one)");
}

#[test]
fn print_print_nested_list() {
    let mut mem = Memory::new();

    let vec1 = vec![mem.allocate_number(2.71), ExternalReference::nil(), mem.allocate_character('$'), mem.symbol_for("puppy")];
    let list1 = vec_to_list(&mut mem, vec1);
    let vec2 = vec![mem.symbol_for("one"), list1, mem.symbol_for("two")];
    let list2 = vec_to_list(&mut mem, vec2);
    let p = print(&mut mem, list2);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "(one (2.71 () %$ puppy) two)");
}

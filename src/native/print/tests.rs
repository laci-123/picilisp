use pretty_assertions::assert_eq;
use super::*;


#[test]
fn print_print_atom() {
    let mut mem = Memory::new();

    let x = GcRef::nil();
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

    let x = mem.allocate_function(GcRef::nil(), FunctionKind::Lambda, vec![]);
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

    let vec = vec![mem.allocate_number(2.71), GcRef::nil(), mem.allocate_character('$'), mem.symbol_for("puppy")];
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

    let vec1 = vec![mem.allocate_number(2.71), GcRef::nil(), mem.allocate_character('$'), mem.symbol_for("puppy")];
    let list1 = vec_to_list(&mut mem, vec1);
    let vec2 = vec![mem.symbol_for("one"), list1, mem.symbol_for("two")];
    let list2 = vec_to_list(&mut mem, vec2);
    let p = print(&mut mem, list2);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "(one (2.71 () %$ puppy) two)");
}

#[test]
fn print_print_string() {
    let mut mem = Memory::new();

    let list = string_to_list(&mut mem, "The quick brown fox jumps over the lazy dog.");
    let p = print(&mut mem, list);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, r#""The quick brown fox jumps over the lazy dog.""#);
}

#[test]
fn print_print_almost_string() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_character('a'), mem.allocate_character('b'), mem.allocate_character('c'), mem.symbol_for("d")];
    let list = vec_to_list(&mut mem, vec);
    let p = print(&mut mem, list);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, "(%a %b %c d)");
}

#[test]
fn print_print_string_in_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_number(1.0), string_to_list(&mut mem, "two"), mem.allocate_number(3.0)];
    let list = vec_to_list(&mut mem, vec);
    let p = print(&mut mem, list);
    let s = list_to_string(p).unwrap();
    assert_eq!(s, r#"(1 "two" 3)"#);
}

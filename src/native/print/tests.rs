use pretty_assertions::assert_eq;
use super::*;


#[test]
fn print_print_atom() {
    let mut mem = Memory::new();

    let x = GcRef::nil();
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "()");

    let x = mem.allocate_number(-123);
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "-123");

    let x = mem.allocate_character('A');
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "%A");

    let x = mem.symbol_for("kitten");
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "kitten");

    let has_rest_params = false;
    let x = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, GcRef::nil(), &vec![], GcRef::nil());
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "#<function>");

    let car = mem.allocate_number(999);
    let cdr = mem.allocate_character('9');
    let x = mem.allocate_cons(car, cdr);
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "(cons 999 %9)");

    let car = GcRef::nil();
    let cdr = mem.allocate_number(1);
    let x = mem.allocate_cons(car, cdr);
    let p = print(&mut mem, &[x], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "(cons () 1)");
}

#[test]
fn print_print_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_number(271), GcRef::nil(), mem.allocate_character('$'), mem.symbol_for("puppy")];
    let list = vec_to_list(&mut mem, &vec);
    let p = print(&mut mem, &[list], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "(271 () %$ puppy)");
}

#[test]
fn print_print_singleton_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.symbol_for("only-this-one")];
    let list = vec_to_list(&mut mem, &vec);
    let p = print(&mut mem, &[list], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "(only-this-one)");
}

#[test]
fn print_print_nested_list() {
    let mut mem = Memory::new();

    let vec1 = vec![mem.allocate_number(271), GcRef::nil(), mem.allocate_character('$'), mem.symbol_for("puppy")];
    let list1 = vec_to_list(&mut mem, &vec1);
    let vec2 = vec![mem.symbol_for("one"), list1, mem.symbol_for("two")];
    let list2 = vec_to_list(&mut mem, &vec2);
    let p = print(&mut mem, &[list2], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "(one (271 () %$ puppy) two)");
}

#[test]
fn print_print_string() {
    let mut mem = Memory::new();

    let list = string_to_list(&mut mem, "The quick brown fox jumps over the lazy dog.");
    let p = print(&mut mem, &[list], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, r#""The quick brown fox jumps over the lazy dog.""#);
}

#[test]
fn print_print_almost_string() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_character('a'), mem.allocate_character('b'), mem.allocate_character('c'), mem.symbol_for("d")];
    let list = vec_to_list(&mut mem, &vec);
    let p = print(&mut mem, &[list], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, "(%a %b %c d)");
}

#[test]
fn print_print_string_in_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_number(1), string_to_list(&mut mem, "two"), mem.allocate_number(3)];
    let list = vec_to_list(&mut mem, &vec);
    let p = print(&mut mem, &[list], GcRef::nil(), 0);
    let s = list_to_string(p.ok().unwrap()).unwrap();
    assert_eq!(s, r#"(1 "two" 3)"#);
}

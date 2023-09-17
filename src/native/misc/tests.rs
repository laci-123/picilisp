use pretty_assertions::assert_eq;
use crate::util::{vec_to_list, assert_eq_symbol};
use super::*;



#[test]
fn euqal_same_type() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(100);
    let y = mem.allocate_number(100);

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq_symbol!(e, mem.symbol_for("t"));

    let x = mem.allocate_number(100);
    let y = mem.allocate_number(200);

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let x = mem.allocate_character('a');
    let y = mem.allocate_character('a');

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq_symbol!(e, mem.symbol_for("t"));

    let x = mem.allocate_character('a');
    let y = mem.allocate_character('b');

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let x1 = mem.allocate_number(-1);
    let y1 = mem.allocate_number(-2);
    let z1 = mem.allocate_cons(x1, y1);
    let x2 = mem.allocate_number(-1);
    let y2 = mem.allocate_number(-2);
    let z2 = mem.allocate_cons(x2, y2);

    let e = equal(&mut mem, &[z1, z2], GcRef::nil(), 0).ok().unwrap();
    assert_eq_symbol!(e, mem.symbol_for("t"));

    let x1 = mem.allocate_number(-1);
    let y1 = mem.allocate_number(-2);
    let z1 = mem.allocate_cons(x1, y1);
    let x2 = mem.allocate_number(-10);
    let y2 = mem.allocate_number(3);
    let z2 = mem.allocate_cons(x2, y2);

    let e = equal(&mut mem, &[z1, z2], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());
}

#[test]
fn euqal_different_type() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_character('1');

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let x = mem.allocate_number(2);
    let y = mem.symbol_for("2");

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());
}

#[test]
fn euqal_nil() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(0);
    let y = GcRef::nil();

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let x = mem.allocate_character('0');
    let y = GcRef::nil();

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let x = mem.symbol_for("nil");
    let y = GcRef::nil();

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());
}

#[test]
fn euqal_nil_with_nil() {
    let mut mem = Memory::new();

    let x = GcRef::nil();
    let y = GcRef::nil();

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq_symbol!(e, mem.symbol_for("t"));
}

#[test]
fn euqal_list() {
    let mut mem = Memory::new();

    let v1 = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let x = vec_to_list(&mut mem, &v1);
    let v2 = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let y = vec_to_list(&mut mem, &v2);

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq_symbol!(e, mem.symbol_for("t"));

    let v1 = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let x = vec_to_list(&mut mem, &v1);
    let v2 = vec![mem.allocate_number(1), mem.allocate_number(20), mem.allocate_number(3)];
    let y = vec_to_list(&mut mem, &v2);

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let v1 = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let x = vec_to_list(&mut mem, &v1);
    let v2 = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3), mem.allocate_number(4)];
    let y = vec_to_list(&mut mem, &v2);

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());

    let v1 = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let x = vec_to_list(&mut mem, &v1);
    let v2 = vec![mem.allocate_number(1), mem.allocate_number(2)];
    let y = vec_to_list(&mut mem, &v2);

    let e = equal(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert!(e.is_nil());
}

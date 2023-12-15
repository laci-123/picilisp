use pretty_assertions::assert_eq;
use crate::util::{vec_to_list, list_to_vec};
use super::*;


#[test]
fn lists_cons() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_number(2);
    let c = cons(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*c.get().unwrap().as_conscell().get_car().get().unwrap().as_number(), 1);
    assert_eq!(*c.get().unwrap().as_conscell().get_cdr().get().unwrap().as_number(), 2);
}

#[test]
fn lists_car() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_number(2);
    let c = mem.allocate_cons(x, y);
    let z = car(&mut mem, &[c], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*z.get().unwrap().as_number(), 1);
}

#[test]
fn lists_cdr() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_number(2);
    let c = mem.allocate_cons(x, y);
    let z = cdr(&mut mem, &[c], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*z.get().unwrap().as_number(), 2);
}

#[test]
fn lists_list() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_number(2);
    let z = mem.allocate_number(3);
    let l = list(&mut mem, &[x, y, z], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*l.get().unwrap().as_conscell().get_car().get().unwrap().as_number(), 1);
    assert_eq!(*l.get().unwrap().as_conscell().get_cdr().get().unwrap().as_conscell().get_car().get().unwrap().as_number(), 2);
    assert_eq!(*l.get().unwrap().as_conscell().get_cdr().get().unwrap().as_conscell().get_cdr().get().unwrap().as_conscell().get_car().get().unwrap().as_number(), 3);
    assert!(    l.get().unwrap().as_conscell().get_cdr().get().unwrap().as_conscell().get_cdr().get().unwrap().as_conscell().get_cdr().is_nil());
}

#[test]
fn append() {
    let mut mem = Memory::new();

    let vec1  = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let list1 = vec_to_list(&mut mem, &vec1);
    let vec2  = vec![mem.symbol_for("a"), mem.symbol_for("b")];
    let list2 = vec_to_list(&mut mem, &vec2);

    let list3 = super::append(&mut mem, &[list1, list2], GcRef::nil(), 0).ok().unwrap();
    let vec3  = list_to_vec(list3).unwrap();
    assert_eq!(vec3.len(), 5);
    assert_eq!(*vec3[0].get().unwrap().as_number(), 1);
    assert_eq!(*vec3[1].get().unwrap().as_number(), 2);
    assert_eq!(*vec3[2].get().unwrap().as_number(), 3);
    assert_eq_symbol!(vec3[3], mem.symbol_for("a"));
    assert_eq_symbol!(vec3[4], mem.symbol_for("b"));
}

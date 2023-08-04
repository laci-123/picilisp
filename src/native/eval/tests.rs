use pretty_assertions::assert_eq;
use super::*;


#[test]
fn lookup_empty() {
    let mut mem = Memory::new();

    let env = ExternalReference::nil();
    let value = lookup(mem.symbol_for("bird"), env);
    assert!(value.is_none());
}

#[test]
fn lookup_not_found() {
    let mut mem = Memory::new();

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(1.0);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(2.0);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, vec);
    let value = lookup(mem.symbol_for("bird"), env);
    assert!(value.is_none());
}

#[test]
fn lookup_found() {
    let mut mem = Memory::new();

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(1.0);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(2.0);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, vec);
    let value = lookup(mem.symbol_for("falcon"), env);
    assert_eq!(*value.unwrap().get().as_number(), 2.0);
}

#[test]
fn eval_number() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_number(365.0);
    let value = eval(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), 365.0);
}

#[test]
fn eval_character() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_character('Đ');
    let value = eval(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_character(), 'Đ');
}

#[test]
fn eval_unbound_symbol() {
    let mut mem = Memory::new();

    let tree  = mem.symbol_for("apple-tree");
    let value = eval(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unbound-symbol");
}

#[test]
fn eval_cons() {
    let mut mem = Memory::new();

    let x     = mem.allocate_number(1.0);
    let y     = mem.allocate_number(2.0);
    let tree  = mem.allocate_cons(x, y);
    let value = eval(&mut mem, tree);
    assert_eq!(*value.clone().unwrap().get().as_conscell().get_car().get().as_number(), 1.0);
    assert_eq!(*value.unwrap().get().as_conscell().get_cdr().get().as_number(), 2.0);
}

use pretty_assertions::assert_eq;
use crate::util::*;
use crate::native::list::property;
use super::*;


#[test]
fn numbers_add() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_number(2);

    let z = add(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*z.get().unwrap().as_number(), 3);
}

#[test]
fn numbers_substract() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1);
    let y = mem.allocate_number(2);

    let z = substract(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*z.get().unwrap().as_number(), -1);
}

#[test]
fn numbers_multiply() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(2);
    let y = mem.allocate_number(4);

    let z = multiply(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*z.get().unwrap().as_number(), 8);
}

#[test]
fn numbers_overflow() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(1000000000000);
    let y = mem.allocate_number(1000000000000);

    let z = multiply(&mut mem, &[x, y], GcRef::nil(), 0).err().unwrap();
    assert_eq_symbol!(property(&mut mem, "kind", z).unwrap(), mem.symbol_for("arithmetic-overflow"));
}

#[test]
fn numbers_devide() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(12);
    let y = mem.allocate_number(3);

    let z = divide(&mut mem, &[x, y], GcRef::nil(), 0).ok().unwrap();
    assert_eq!(*z.get().unwrap().as_number(), 4);
}

#[test]
fn numbers_devide_by_zero() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(12);
    let y = mem.allocate_number(0);

    let z = divide(&mut mem, &[x, y], GcRef::nil(), 0).err().unwrap();
    assert_eq_symbol!(property(&mut mem, "kind", z).unwrap(), mem.symbol_for("divide-by-zero"));
}

use pretty_assertions::assert_eq;
use super::*;


#[test]
fn core_vec_to_list_reverse() {
    let mut mem = Memory::new();

    let vec = vec![mem.allocate_number(1.0),
                   mem.allocate_number(2.0),
                   mem.allocate_number(3.0)];

    let list = vec_to_list_reverse(&mut mem, vec);

    let mut c = list;
    let c1 = c.get().as_conscell();
    let x1 = *c1.get_car().get().as_number();
    assert_eq!(x1, 3.0);
    c = c1.get_cdr();
    let c2 = c.get().as_conscell();
    let x2 = *c2.get_car().get().as_number();
    assert_eq!(x2, 2.0);
    c = c2.get_cdr();
    let c3 = c.get().as_conscell();
    let x3 = *c3.get_car().get().as_number();
    assert_eq!(x3, 1.0);
    c = c3.get_cdr();
    assert!(c.is_nil());
}

#[test]
fn core_vec_to_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.symbol_for("A"),
                   mem.symbol_for("B"),
                   mem.symbol_for("C")];

    let list = vec_to_list(&mut mem, vec);

    let mut c = list;
    let c1 = c.get().as_conscell();
    assert_eq!(c1.get_car().get().as_symbol(), mem.symbol_for("A").get().as_symbol());
    c = c1.get_cdr();
    let c2 = c.get().as_conscell();
    assert_eq!(c2.get_car().get().as_symbol(), mem.symbol_for("B").get().as_symbol());
    c = c2.get_cdr();
    let c3 = c.get().as_conscell();
    assert_eq!(c3.get_car().get().as_symbol(), mem.symbol_for("C").get().as_symbol());
    c = c3.get_cdr();
    assert!(c.is_nil());
}

#[test]
fn core_list_to_vec() {
    let mut mem = Memory::new();

    // empty list
    let list = ExternalReference::nil();
    assert_eq!(list_to_vec(list).unwrap().len(), 0);
    
    // non-empty list
    let mut list = ExternalReference::nil();
    for i in 0 .. 5 {
        let x = mem.allocate_number(i as f64);
        list = mem.allocate_cons(x, list);
    }

    let vec = list_to_vec(list);

    for (i, x) in vec.unwrap().iter().enumerate() {
        assert_eq!(*x.get().as_number(), i as f64);
    }
}

#[test]
fn core_list_to_vec_fail() {
    let mut mem = Memory::new();

    // not list at all
    let not_list = mem.allocate_character('@');
    assert!(list_to_vec(not_list).is_none());

    // not valid list
    let x1 = mem.allocate_number(10.0);
    let x2 = mem.allocate_number(20.0);
    let x3 = mem.allocate_number(30.0);
    let c1 = mem.allocate_cons(x2, x3);
    let c2 = mem.allocate_cons(x1, c1);
    assert!(list_to_vec(c2).is_none());
}

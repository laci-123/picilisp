use pretty_assertions::assert_eq;
use super::*;


#[test]
fn util_vec_to_list() {
    let mut mem = Memory::new();

    let vec = vec![mem.symbol_for("A"),
                   mem.symbol_for("B"),
                   mem.symbol_for("C")];

    let list = vec_to_list(&mut mem, &vec);

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
fn util_string_to_list() {
    let mut mem = Memory::new();
    
    let list = string_to_list(&mut mem, "cat");

    let mut c = list;
    let c1 = c.get().as_conscell();
    assert_eq!(*c1.get_car().get().as_character(), 'c');
    c = c1.get_cdr();                             
    let c2 = c.get().as_conscell();
    assert_eq!(*c2.get_car().get().as_character(), 'a');
    c = c2.get_cdr();
    let c3 = c.get().as_conscell();
    assert_eq!(*c3.get_car().get().as_character(), 't');
    c = c3.get_cdr();
    assert!(c.is_nil());
}


#[test]
fn util_vec_to_list_empty() {
    let mut mem = Memory::new();
    
    let vec = vec![];
    let list = vec_to_list(&mut mem, &vec);
    assert!(list.is_nil());
}


#[test]
fn util_list_to_vec() {
    let mut mem = Memory::new();

    // empty list
    let list = GcRef::nil();
    assert_eq!(list_to_vec(list).unwrap().len(), 0);
    
    // non-empty list
    let mut list = GcRef::nil();
    for i in (0 .. 5).rev() {
        let x = mem.allocate_number(i as f64);
        list = mem.allocate_cons(x, list);
    }

    let vec = list_to_vec(list);

    for (i, x) in vec.unwrap().iter().enumerate() {
        assert_eq!(*x.get().as_number(), i as f64);
    }
}

#[test]
fn util_list_to_vec_fail() {
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

#[test]
fn util_append_lists() {
    let mut mem = Memory::new();

    let vec1  = vec![mem.allocate_number(11.0), mem.allocate_number(12.0), mem.allocate_number(13.0)];
    let list1 = vec_to_list(&mut mem, &vec1);
    let vec2  = vec![mem.allocate_number(14.0), mem.allocate_number(15.0), mem.allocate_number(16.0)];
    let list2 = vec_to_list(&mut mem, &vec2);

    let list3 = append_lists(&mut mem, list1, list2).unwrap();
    let vec3  = list_to_vec(list3).unwrap();
    assert_eq!(vec3.iter().map(|x| *x.get().as_number()).collect::<Vec<f64>>(), vec![11.0, 12.0, 13.0, 14.0, 15.0, 16.0]);
}



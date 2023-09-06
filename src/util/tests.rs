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
    let c1 = c.get().unwrap().as_conscell();
    assert_eq!(c1.get_car().get().unwrap().as_symbol(), mem.symbol_for("A").get().unwrap().as_symbol());
    c = c1.get_cdr();
    let c2 = c.get().unwrap().as_conscell();
    assert_eq!(c2.get_car().get().unwrap().as_symbol(), mem.symbol_for("B").get().unwrap().as_symbol());
    c = c2.get_cdr();
    let c3 = c.get().unwrap().as_conscell();
    assert_eq!(c3.get_car().get().unwrap().as_symbol(), mem.symbol_for("C").get().unwrap().as_symbol());
    c = c3.get_cdr();
    assert!(c.is_nil());
}

#[test]
fn util_string_to_list() {
    let mut mem = Memory::new();
    
    let list = string_to_list(&mut mem, "cat");

    let mut c = list;
    let c1 = c.get().unwrap().as_conscell();
    assert_eq!(*c1.get_car().get().unwrap().as_character(), 'c');
    c = c1.get_cdr();                             
    let c2 = c.get().unwrap().as_conscell();
    assert_eq!(*c2.get_car().get().unwrap().as_character(), 'a');
    c = c2.get_cdr();
    let c3 = c.get().unwrap().as_conscell();
    assert_eq!(*c3.get_car().get().unwrap().as_character(), 't');
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
        let x = mem.allocate_number(i as i64);
        list = mem.allocate_cons(x, list);
    }

    let vec = list_to_vec(list);

    for (i, x) in vec.unwrap().iter().enumerate() {
        assert_eq!(*x.get().unwrap().as_number(), i as i64);
    }
}

#[test]
fn util_list_to_vec_fail() {
    let mut mem = Memory::new();

    // not list at all
    let not_list = mem.allocate_character('@');
    assert!(list_to_vec(not_list).is_none());

    // not valid list
    let x1 = mem.allocate_number(100);
    let x2 = mem.allocate_number(200);
    let x3 = mem.allocate_number(300);
    let c1 = mem.allocate_cons(x2, x3);
    let c2 = mem.allocate_cons(x1, c1);
    assert!(list_to_vec(c2).is_none());
}

#[test]
fn util_append_lists() {
    let mut mem = Memory::new();

    let vec1  = vec![mem.allocate_number(110), mem.allocate_number(120), mem.allocate_number(130)];
    let list1 = vec_to_list(&mut mem, &vec1);
    let vec2  = vec![mem.allocate_number(140), mem.allocate_number(150), mem.allocate_number(160)];
    let list2 = vec_to_list(&mut mem, &vec2);

    let list3 = append_lists(&mut mem, list1, list2).unwrap();
    let vec3  = list_to_vec(list3).unwrap();
    assert_eq!(vec3.iter().map(|x| *x.get().unwrap().as_number()).collect::<Vec<i64>>(), vec![110, 120, 130, 140, 150, 160]);
}

#[test]
fn util_list_iter_empty() {
    let list = GcRef::nil();

    let mut iterator = list_iter(list);

    assert!(iterator.next().is_none());
}

#[test]
fn util_list_iter() {
    let mut mem = Memory::new();

    let vec  = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let list = vec_to_list(&mut mem, &vec);

    let mut iterator = list_iter(list);

    assert_eq!(*iterator.next().unwrap().unwrap().get().unwrap().as_number(), 1);
    assert_eq!(*iterator.next().unwrap().unwrap().get().unwrap().as_number(), 2);
    assert_eq!(*iterator.next().unwrap().unwrap().get().unwrap().as_number(), 3);
    assert!(iterator.next().is_none());
}

#[test]
fn util_list_iter_invalid() {
    let mut mem = Memory::new();

    let x1 = mem.allocate_character('a');
    let x2 = mem.allocate_character('b');
    let x3 = mem.symbol_for("not-nil");
    let cons1 = mem.allocate_cons(x2, x3);
    let cons2 = mem.allocate_cons(x1, cons1);

    let mut iterator = list_iter(cons2);

    assert_eq!(*iterator.next().unwrap().unwrap().get().unwrap().as_character(), 'a');
    assert_eq!(*iterator.next().unwrap().unwrap().get().unwrap().as_character(), 'b');
    assert!(iterator.next().unwrap().is_none());
}

#[test]
fn util_headstails_iter_empty() {
    let list = GcRef::nil();

    let mut iterator = list_iter_heads_tails(list);

    assert!(iterator.next().is_none());
}

#[test]
fn util_headstails_iter() {
    let mut mem = Memory::new();

    let vec  = vec![mem.allocate_number(1), mem.allocate_number(2), mem.allocate_number(3)];
    let list = vec_to_list(&mut mem, &vec);

    let mut iterator = list_iter_heads_tails(list);

    let (h1, t1) = iterator.next().unwrap().unwrap();
    assert_eq!(*h1.get().unwrap().as_number(), 1);
    let v1 = list_to_vec(t1).unwrap();
    assert_eq!(v1.len(), 2);
    assert_eq!(*v1[0].get().unwrap().as_number(), 2);
    assert_eq!(*v1[1].get().unwrap().as_number(), 3);

    let (h2, t2) = iterator.next().unwrap().unwrap();
    assert_eq!(*h2.get().unwrap().as_number(), 2);
    let v2 = list_to_vec(t2).unwrap();
    assert_eq!(v2.len(), 1);
    assert_eq!(*v2[0].get().unwrap().as_number(), 3);

    let (h3, t3) = iterator.next().unwrap().unwrap();
    assert_eq!(*h3.get().unwrap().as_number(), 3);
    assert!(t3.is_nil());

    assert!(iterator.next().is_none());
}

#[test]
fn util_heads_tails_iter_invalid() {
    let mut mem = Memory::new();

    let x1 = mem.allocate_character('a');
    let x2 = mem.allocate_character('b');
    let x3 = mem.symbol_for("not-nil");
    let cons1 = mem.allocate_cons(x2, x3);
    let cons2 = mem.allocate_cons(x1, cons1);

    let mut iterator = list_iter_heads_tails(cons2);

    let (h1, t1) = iterator.next().unwrap().unwrap();
    assert_eq!(*h1.get().unwrap().as_character(), 'a');
    assert_eq!(*t1.get().unwrap().as_conscell().get_car().get().unwrap().as_character(), 'b');
    assert_eq!(t1.get().unwrap().as_conscell().get_cdr().get().unwrap().as_symbol(), mem.symbol_for("not-nil").get().unwrap().as_symbol());

    let (h2, t2) = iterator.next().unwrap().unwrap();
    assert_eq!(*h2.get().unwrap().as_character(), 'b');
    assert_eq!(t2.get().unwrap().as_symbol(), mem.symbol_for("not-nil").get().unwrap().as_symbol());

    assert!(iterator.next().unwrap().is_none());
}

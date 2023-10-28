use pretty_assertions::assert_eq;
use super::*;



#[test]
fn util_cons_type() {
    let mut mem = Memory::new();

    // empty list
    let list = GcRef::nil();
    let ct = cons_type(list);
    assert!(ct.is_string);
    assert!(ct.is_list);
    
    // non-empty list but not string
    let mut list = GcRef::nil();
    for i in 0..5 {
        let x = mem.allocate_number(i as i64);
        list = mem.allocate_cons(x, list);
    }
    let ct = cons_type(list);
    assert!(!ct.is_string);
    assert!(ct.is_list);

    // non-empty string
    let mut list = GcRef::nil();
    for i in 'a'..'f' {
        let c = mem.allocate_character(i);
        list = mem.allocate_cons(c, list);
    }
    let ct = cons_type(list);
    assert!(ct.is_string);
    assert!(ct.is_list);

    // not even a list
    let x = mem.allocate_character('x');
    let y = mem.allocate_character('y');
    let c = mem.allocate_cons(x, y);
    let ct = cons_type(c);
    assert!(!ct.is_string);
    assert!(!ct.is_list);
}


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
fn util_string_iterator_empty() {
    let mut it = StringIterator::new(GcRef::nil());

    assert!(it.next().is_none());
}

#[test]
fn util_string_iterator() {
    let mut mem = Memory::new();
    
    let s = string_to_list(&mut mem, "öőüű");
    let mut it = StringIterator::new(s);

    let (c, r) = it.next().unwrap().unwrap();
    assert_eq!(c, 'ö');
    assert_eq!(list_to_string(r).unwrap(), "őüű");
    let (c, r) = it.next().unwrap().unwrap();
    assert_eq!(c, 'ő');
    assert_eq!(list_to_string(r).unwrap(), "üű");
    let (c, r) = it.next().unwrap().unwrap();
    assert_eq!(c, 'ü');
    assert_eq!(list_to_string(r).unwrap(), "ű");
    let (c, r) = it.next().unwrap().unwrap();
    assert_eq!(c, 'ű');
    assert_eq!(list_to_string(r).unwrap(), "");
    assert!(it.next().is_none());
}

#[test]
fn util_string_iterator_invalid_string() {
    let mut mem = Memory::new();
    
    let vec = vec![mem.allocate_character('a'), mem.allocate_character('b'), mem.allocate_number(3), mem.allocate_character('d')];
    let s   = vec_to_list(&mut mem, &vec);
    let mut it = StringIterator::new(s);

    let (c, _) = it.next().unwrap().unwrap();
    assert_eq!(c, 'a');
    let (c, _) = it.next().unwrap().unwrap();
    assert_eq!(c, 'b');
    assert!(it.next().unwrap().is_none());
}

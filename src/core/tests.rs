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
fn core_vec_to_list_empty() {
    let mut mem = Memory::new();
    
    let vec = vec![];
    let list = vec_to_list(&mut mem, vec);
    assert!(list.is_nil());
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

#[test]
fn fold_tree_atom() {
    let mut mem = Memory::new();

    // sum all numbers in tree
    let f_atom = |mem: &mut Memory, _state, atom: ExternalReference| if let PrimitiveValue::Number(_) = atom.get() { atom } else { mem.allocate_number(0.0) };
    let f_list = |mem: &mut Memory, _state, list: Vec<ExternalReference>| FoldOutput::Return(mem.allocate_number(list.iter().map(|x| x.get().as_number()).sum()));

    let tree  = mem.allocate_character('z');
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), 0.0);

    let tree  = mem.allocate_number(3.0);
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), 3.0);
}

#[test]
fn fold_tree_list() {
    let mut mem = Memory::new();

    // sums all numbers in tree
    let f_atom = |mem: &mut Memory, _state, atom: ExternalReference| if let PrimitiveValue::Number(_) = atom.get() { atom } else { mem.allocate_number(0.0) };
    let f_list = |mem: &mut Memory, _state, list: Vec<ExternalReference>| FoldOutput::Return(mem.allocate_number(list.iter().map(|x| x.get().as_number()).sum()));

    // empty list
    let tree  = ExternalReference::nil();
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), 0.0);

    // non-empty list
    let vec   = vec![mem.allocate_number(1.0), mem.allocate_number(2.0), mem.allocate_character('x'), mem.allocate_number(3.0)];
    let tree  = vec_to_list(&mut mem, vec);
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), 6.0);
}


#[test]
fn fold_tree_nested_list() {
    let mut mem = Memory::new();

    // sums all numbers in tree
    let f_atom = |mem: &mut Memory, _state, atom: ExternalReference| if let PrimitiveValue::Number(_) = atom.get() { atom } else { mem.allocate_number(0.0) };
    let f_list = |mem: &mut Memory, _state, list: Vec<ExternalReference>| FoldOutput::Return(mem.allocate_number(list.iter().map(|x| x.get().as_number()).sum()));

    let vec1  = vec![mem.allocate_number(1.0), mem.symbol_for("something"), mem.allocate_number(2.0), mem.allocate_number(3.0)];
    let list1 = vec_to_list(&mut mem, vec1);
    let vec2  = vec![mem.allocate_number(4.0), mem.allocate_number(5.0), mem.allocate_number(6.0)];
    let list2 = vec_to_list(&mut mem, vec2);
    let vec   = vec![mem.allocate_character('?'), list1, mem.allocate_number(7.0), mem.allocate_number(8.0), ExternalReference::nil(), list2, mem.allocate_number(9.0)];
    let tree  = vec_to_list(&mut mem, vec);
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), 45.0);
}

#[test]
fn fold_tree_signal() {
    let mut mem = Memory::new();

    // sums all numbers in tree, signals if found a negative number
    let f_atom = |mem: &mut Memory, _state, atom: ExternalReference| if let PrimitiveValue::Number(_) = atom.get() { atom } else { mem.allocate_number(0.0) };
    let f_list = |mem: &mut Memory, _state, list: Vec<ExternalReference>| {
        let mut sum = 0.0;
        for xr in list {
            let x = xr.get().as_number();
            if *x < 0.0 {
                return FoldOutput::Signal(mem.symbol_for("found-a-negative"));
            }
            sum += *x;
        }

        FoldOutput::Return(mem.allocate_number(sum))
    };

    // without trap
    let vec1  = vec![mem.allocate_number(1.0), mem.symbol_for("something"), mem.allocate_number(2.0), mem.allocate_number(3.0)];
    let list1 = vec_to_list(&mut mem, vec1);                       ////
    let vec2  = vec![mem.allocate_number(4.0), mem.allocate_number(-5.0), mem.allocate_number(6.0)];
    let list2 = vec_to_list(&mut mem, vec2);
    let vec   = vec![mem.allocate_character('?'), list1, mem.allocate_number(7.0), mem.allocate_number(8.0), ExternalReference::nil(), list2, mem.allocate_number(9.0)];
    let tree  = vec_to_list(&mut mem, vec);
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(sum.as_signal().get().as_symbol(), mem.symbol_for("found-a-negative").get().as_symbol());

    // with trap
    let vec1  = vec![mem.allocate_number(1.0), mem.symbol_for("something"), mem.allocate_number(2.0), mem.allocate_number(3.0)];
    let list1 = vec_to_list(&mut mem, vec1);                       ////
    let vec2  = vec![mem.allocate_number(4.0), mem.allocate_number(-5.0), mem.allocate_number(6.0)];
    let list2 = vec_to_list(&mut mem, vec2);
    let vec   = vec![mem.allocate_character('?'), list1, mem.allocate_number(7.0), mem.allocate_number(8.0), ExternalReference::nil(), list2, mem.allocate_number(9.0)];
    let normal_tree = vec_to_list(&mut mem, vec);
    let trap_tree   = mem.allocate_number(-123.0);
    let tree  = mem.allocate_trap(normal_tree, trap_tree);
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), -123.0);

    // with trap but with no signaling
    let vec1  = vec![mem.allocate_number(1.0), mem.symbol_for("something"), mem.allocate_number(2.0), mem.allocate_number(3.0)];
    let list1 = vec_to_list(&mut mem, vec1);                       ////
    let vec2  = vec![mem.allocate_number(4.0), mem.allocate_number(5.0), mem.allocate_number(6.0)];
    let list2 = vec_to_list(&mut mem, vec2);
    let vec   = vec![mem.allocate_character('?'), list1, mem.allocate_number(7.0), mem.allocate_number(8.0), ExternalReference::nil(), list2, mem.allocate_number(9.0)];
    let normal_tree = vec_to_list(&mut mem, vec);
    let trap_tree   = mem.allocate_number(-123.0);
    let tree  = mem.allocate_trap(normal_tree, trap_tree);
    let state = ExternalReference::nil();
    let sum   = fold_tree(&mut mem, state, tree, f_atom, f_list);
    assert_eq!(*sum.as_value().get().as_number(), 45.0);
}


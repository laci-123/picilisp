use pretty_assertions::{assert_eq, assert_ne};
use super::*;


#[test]
fn memory_init() {
    let mem = Memory::new();

    assert_eq!(mem.free_cells.len(), INITIAL_FREE_CELLS);
    assert!(mem.free_cells.iter().all(|c| c.content.value.is_nil()));
}

#[test]
fn memory_allocate() {
    let mut mem = Memory::new();

    {
        let mut refs = vec![]; // keep references -> prevent garbage collection
        for i in 0 .. INITIAL_FREE_CELLS {
            refs.push(mem.allocate_number(i as f64));
        }

        assert_eq!(mem.free_cells.len(), 0);
        assert_eq!(mem.used_cells.len(), INITIAL_FREE_CELLS);

        refs.push(mem.allocate_number(-1.0));

        assert_eq!(mem.free_cells.len(), ALLOCATION_INCREMENT - 1);
        assert_eq!(mem.used_cells.len(), INITIAL_FREE_CELLS + 1);

        for i in 0 .. 100 {
            refs.push(mem.allocate_number(i as f64));
        }

        // drop refs
    }

    // make sure gc collect actually happens
    for i in 100 .. 200 {
        mem.allocate_number(i as f64);
    }

    assert_eq!(mem.used_cells.len(), 1);
}

#[test]
fn externalreference_deref() {
    let mut mem = Memory::new();

    let reference1 = mem.allocate_number(3.1416);
    let reference2 = mem.allocate_character('Q');

    assert_eq!(*reference1.get().as_number(), 3.1416);
    assert_eq!(*reference2.get().as_character(), 'Q');
}

#[test]
fn memory_allocate_cons() {
    let mut mem = Memory::new();
    let length = 10;

    {
        let mut c = ExternalRefrence::nil();
        for i in 0 .. length {
            let x = mem.allocate_number(i as f64);
            c = mem.allocate_cons(x, c);
        }

        let mut i = length;
        while !c.is_nil() {
            i -= 1;
            assert_eq!(*c.get().as_conscell().get_car().get().as_number(), i as f64);

            c = c.get().as_conscell().get_cdr();
        }

        assert_eq!(mem.used_cells.len(), 2 * length);
    }

    mem.collect();

    assert_eq!(mem.used_cells.len(), 0);
}

#[test]
fn memory_allocate_symbol() {
    let mut mem = Memory::new();
    
    let sym1 = mem.symbol_for("elephant");
    let sym2 = mem.symbol_for("mamoth");
    let sym3 = mem.symbol_for("elephant");

    assert_eq!(sym1.get().as_symbol(), sym3.get().as_symbol());
    assert_ne!(sym1.get().as_symbol(), sym2.get().as_symbol());
    assert_ne!(sym3.get().as_symbol(), sym2.get().as_symbol());

    assert_eq!(mem.used_cells.len(), 2);
}

#[test]
fn memory_allocate_unique_symbol() {
    let mut mem = Memory::new();
    
    let sym1 = mem.unique_symbol();
    let sym2 = mem.symbol_for("whale");
    let sym3 = mem.unique_symbol();

    assert_ne!(sym1.get().as_symbol(), sym2.get().as_symbol());
    assert_ne!(sym1.get().as_symbol(), sym3.get().as_symbol());
    assert_ne!(sym2.get().as_symbol(), sym3.get().as_symbol());

    assert_eq!(mem.used_cells.len(), 3);
}

#[test]
fn gc_collect_symbols() {
    let mut mem = Memory::new();
    
    let sym1 = mem.symbol_for("cat");
    {
        let mut syms = vec![];
        for i in 0 .. 10 {
            syms.push(mem.symbol_for(&format!("{i}")));
        }
    }
    let sym2 = mem.symbol_for("cat");

    mem.collect();

    assert_eq!(sym1.get().as_symbol(), sym2.get().as_symbol());

    assert_eq!(mem.used_cells.len(), 1);
    assert_eq!(mem.symbols.len(), 1);
}


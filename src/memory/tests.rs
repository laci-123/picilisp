use pretty_assertions::{assert_eq, assert_ne};
use super::*;


#[test]
fn memory_init() {
    let mem = Memory::new();

    assert_eq!(mem.free_count(), INITIAL_FREE_CELLS);
    for i in mem.first_free .. mem.cells.len() {
        assert!(mem.cells[i].content.value.is_nil());
    }
}

#[test]
fn memory_allocate() {
    let mut mem = Memory::new();

    {
        let mut refs = vec![]; // keep references -> prevent garbage collection
        for i in 0 .. INITIAL_FREE_CELLS {
            refs.push(mem.allocate_number(i as f64));
        }

        assert_eq!(mem.free_count(), 0);
        assert_eq!(mem.used_count(), INITIAL_FREE_CELLS);

        refs.push(mem.allocate_number(-1.0));

        assert_eq!(mem.free_count(), ALLOCATION_INCREMENT - 1);
        assert_eq!(mem.used_count(), INITIAL_FREE_CELLS + 1);

        for i in 0 .. 100 {
            refs.push(mem.allocate_number(i as f64));
        }

        // drop refs
    }

    // make sure gc collect actually happens
    for i in 100 .. 200 {
        mem.allocate_number(i as f64);
    }

    assert_eq!(mem.used_count(), 1);
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
fn externalreference_clone() {
    let mut mem = Memory::new();

    let r4;
    {
        let r1 = mem.allocate_number(-1.2);
        let r2 = r1.clone();
        let r3 = mem.allocate_character('1');
        r4 = r3.clone();

        assert_eq!(*r2.get().as_number(), -1.2);
        assert_eq!(mem.used_count(), 2);
    }
    assert_eq!(*r4.get().as_character(), '1');

    mem.collect();

    assert_eq!(mem.used_count(), 1);
}

#[test]
fn memory_allocate_cons() {
    let mut mem = Memory::new();
    let length = 10;

    let remain = mem.allocate_number(9.9); // this one should not get garbage collected
    
    {
        let mut c = GcRef::nil();
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

        assert_eq!(mem.used_count(), 2 * length + 1);
    }

    mem.collect();

    assert_eq!(mem.used_count(), 1);
    assert_eq!(*remain.get().as_number(), 9.9);
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

    assert_eq!(mem.used_count(), 2);
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

    assert_eq!(mem.used_count(), 3);
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

    assert_eq!(mem.used_count(), 1);
    assert_eq!(mem.symbols.len(), 1);
}

#[test]
fn mem_allocate_function() {
    let mut mem = Memory::new();
    
    let p1 = mem.symbol_for("oak");
    let p2 = mem.symbol_for("pine");
    let p3 = mem.symbol_for("elm");
    let body = mem.allocate_character('ß');

    let fun = mem.allocate_normal_function(FunctionKind::Lambda, body, vec![p1, p2, p3]);
    assert_eq!(*fun.get().as_function().as_normal_function().get_body().get().as_character(), 'ß');
    let mut params = fun.get().as_function().as_normal_function().params();
    assert_eq!(params.next().unwrap().get().as_symbol(), mem.symbol_for("oak").get().as_symbol());
    assert_eq!(params.next().unwrap().get().as_symbol(), mem.symbol_for("pine").get().as_symbol());
    assert_eq!(params.next().unwrap().get().as_symbol(), mem.symbol_for("elm").get().as_symbol());
    assert!(params.next().is_none());
    assert_eq!(fun.get().as_function().as_normal_function().kind, FunctionKind::Lambda);
}

#[test]
fn gc_collect_functions() {
    let mut mem = Memory::new();

    {
        mem.allocate_number(0.0);

        let p1 = mem.symbol_for("tulip");
        let p2 = mem.symbol_for("rose");
        let p3 = mem.symbol_for("sunflower");
        let body = mem.allocate_character('🌻');

        let fun = mem.allocate_normal_function(FunctionKind::Macro, body, vec![p1, p2, p3]);

        mem.symbol_for("tulip");

        assert_eq!(fun.get().as_function().as_normal_function().params().collect::<Vec<_>>().len(), 3);
    }

    mem.collect();

    assert_eq!(mem.used_count(), 0);
}

#[test]
fn mem_allocate_trap() {
    let mut mem = Memory::new();
    
    {
        let x = mem.allocate_number(100.2);
        let y = mem.allocate_character('a');
        let trap = mem.allocate_trap(x, y);

        assert_eq!(*trap.get().as_trap().get_normal_body().get().as_number(), 100.2);
        assert_eq!(*trap.get().as_trap().get_trap_body().get().as_character(), 'a');
    }

    mem.collect();

    assert_eq!(mem.used_count(), 0);
}

#[test]
fn mem_allocate_meta() {
    let mut mem = Memory::new();
    
    {
        let x1   = mem.allocate_number(137.0);
        let loc1 = Location::in_stdin(23, 24);
        let m1   = mem.allocate_metadata(x1, loc1);

        let x2   = mem.allocate_character(' ');
        let loc2 = Location::in_file(Path::new("~/the/input/file.lisp"), 41, 42);
        let m2   = mem.allocate_metadata(x2, loc2);

        let y    = mem.symbol_for("thing");

        assert_eq!(*m1.get().as_number(),    137.0);
        assert_eq!(*m2.get().as_character(), ' ');

        assert_eq!(m1.get_metadata().unwrap().file, None);
        assert_eq!(m1.get_metadata().unwrap().line, 23);
        assert_eq!(m2.get_metadata().unwrap().file.as_ref().unwrap(), Path::new("~/the/input/file.lisp"));
        assert_eq!(m2.get_metadata().unwrap().column, 42);

        assert_eq!(y.get_metadata(), None);
    }

    mem.collect();

    assert_eq!(mem.used_count(), 0);
}

#[test]
fn mem_globals() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(14.0);
    mem.define_global("x", x);
    let y = mem.symbol_for("thing");
    mem.define_global("y", y);

    assert_eq!(*mem.get_global("x").unwrap().get().as_number(), 14.0);
    assert!(mem.get_global("z").is_none());

    mem.undefine_global("x");
    assert!(mem.get_global("x").is_none());

    mem.collect();

    assert_eq!(mem.get_global("y").unwrap().get().as_symbol(), mem.symbol_for("thing").get().as_symbol());
}

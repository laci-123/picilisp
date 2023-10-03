use pretty_assertions::{assert_eq, assert_ne};
use crate::config;
use std::path::PathBuf;
use super::*;


#[test]
fn memory_init() {
    let mem = Memory::new();

    assert_eq!(mem.free_count(), config::INITIAL_FREE_CELLS);
    for i in mem.first_free .. mem.cells.len() {
        assert!(mem.cells[i].content.value.is_nil());
    }
}

#[test]
fn memory_allocate() {
    let mut mem = Memory::new();

    {
        let mut refs = vec![]; // keep references -> prevent garbage collection
        for i in 0 .. config::INITIAL_FREE_CELLS {
            refs.push(mem.allocate_number(i as i64));
        }

        assert_eq!(mem.free_count(), 0);
        assert_eq!(mem.used_count(), config::INITIAL_FREE_CELLS);

        refs.push(mem.allocate_number(-10));

        assert_eq!(mem.free_count() as f32, mem.used_count() as f32 * config::ALLOCATION_RATIO - 1.0);
        assert_eq!(mem.used_count(), config::INITIAL_FREE_CELLS + 1);

        for i in 0 .. 100 {
            refs.push(mem.allocate_number(i as i64));
        }

        // drop refs
    }

    // make sure gc collect actually happens
    for i in 100 .. 1000 {
        mem.allocate_number(i as i64);
    }

    assert_eq!(mem.used_count(), 1);
}

#[test]
fn externalreference_deref() {
    let mut mem = Memory::new();

    let reference1 = mem.allocate_number(31416);
    let reference2 = mem.allocate_character('Q');

    assert_eq!(*reference1.get().unwrap().as_number(), 31416);
    assert_eq!(*reference2.get().unwrap().as_character(), 'Q');
}

#[test]
fn externalreference_clone() {
    let mut mem = Memory::new();

    let r4;
    {
        let r1 = mem.allocate_number(-12);
        let r2 = r1.clone();
        let r3 = mem.allocate_character('1');
        r4 = r3.clone();

        assert_eq!(*r2.get().unwrap().as_number(), -12);
        assert_eq!(mem.used_count(), 2);
    }
    assert_eq!(*r4.get().unwrap().as_character(), '1');

    mem.collect();

    assert_eq!(mem.used_count(), 1);
}

#[test]
fn memory_allocate_cons() {
    let mut mem = Memory::new();
    let length = 10;

    let remain = mem.allocate_number(99); // this one should not get garbage collected
    
    {
        let mut c = GcRef::nil();
        for i in 0 .. length {
            let x = mem.allocate_number(i as i64);
            c = mem.allocate_cons(x, c);
        }

        let mut i = length;
        while !c.is_nil() {
            i -= 1;
            assert_eq!(*c.get().unwrap().as_conscell().get_car().get().unwrap().as_number(), i as i64);

            c = c.get().unwrap().as_conscell().get_cdr();
        }

        assert_eq!(mem.used_count(), 2 * length + 1);
    }

    mem.collect();

    assert_eq!(mem.used_count(), 1);
    assert_eq!(*remain.get().unwrap().as_number(), 99);
}

#[test]
fn memory_allocate_symbol() {
    let mut mem = Memory::new();
    
    let sym1 = mem.symbol_for("elephant");
    let sym2 = mem.symbol_for("mamoth");
    let sym3 = mem.symbol_for("elephant");

    assert_eq!(sym1.get().unwrap().as_symbol(), sym3.get().unwrap().as_symbol());
    assert_ne!(sym1.get().unwrap().as_symbol(), sym2.get().unwrap().as_symbol());
    assert_ne!(sym3.get().unwrap().as_symbol(), sym2.get().unwrap().as_symbol());

    assert_eq!(mem.used_count(), 2);
}

#[test]
fn memory_allocate_unique_symbol() {
    let mut mem = Memory::new();
    
    let sym1 = mem.unique_symbol();
    let sym2 = mem.symbol_for("whale");
    let sym3 = mem.unique_symbol();

    assert_ne!(sym1.get().unwrap().as_symbol(), sym2.get().unwrap().as_symbol());
    assert_ne!(sym1.get().unwrap().as_symbol(), sym3.get().unwrap().as_symbol());
    assert_ne!(sym2.get().unwrap().as_symbol(), sym3.get().unwrap().as_symbol());

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

    assert_eq!(sym1.get().unwrap().as_symbol(), sym2.get().unwrap().as_symbol());

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

    let has_rest_params = false;
    let fun = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, body, &vec![p1, p2, p3], GcRef::nil());
    assert_eq!(*fun.get().unwrap().as_function().as_normal_function().get_body().get().unwrap().as_character(), 'ß');
    let mut params = fun.get().unwrap().as_function().as_normal_function().non_rest_params();
    assert_eq!(params.next().unwrap().get().unwrap().as_symbol(), mem.symbol_for("oak").get().unwrap().as_symbol());
    assert_eq!(params.next().unwrap().get().unwrap().as_symbol(), mem.symbol_for("pine").get().unwrap().as_symbol());
    assert_eq!(params.next().unwrap().get().unwrap().as_symbol(), mem.symbol_for("elm").get().unwrap().as_symbol());
    assert!(params.next().is_none());
    assert_eq!(fun.get().unwrap().as_function().as_normal_function().kind, FunctionKind::Lambda);
}

#[test]
fn gc_collect_functions() {
    let mut mem = Memory::new();

    {
        mem.allocate_number(00);

        let p1 = mem.symbol_for("tulip");
        let p2 = mem.symbol_for("rose");
        let p3 = mem.symbol_for("sunflower");
        let body = mem.allocate_character('🌻');

        let has_rest_params = false;
        let fun = mem.allocate_normal_function(FunctionKind::Macro, has_rest_params, body, &vec![p1, p2, p3], GcRef::nil());

        mem.symbol_for("tulip");

        assert_eq!(fun.get().unwrap().as_function().as_normal_function().non_rest_params().collect::<Vec<_>>().len(), 3);
    }

    mem.collect();

    assert_eq!(mem.used_count(), 0);
}

#[test]
fn mem_allocate_trap() {
    let mut mem = Memory::new();
    
    {
        let x = mem.allocate_number(1002);
        let y = mem.allocate_character('a');
        let trap = mem.allocate_trap(x, y);

        assert_eq!(*trap.get().unwrap().as_trap().get_normal_body().get().unwrap().as_number(), 1002);
        assert_eq!(*trap.get().unwrap().as_trap().get_trap_body().get().unwrap().as_character(), 'a');
    }

    mem.collect();

    assert_eq!(mem.used_count(), 0);
}

#[test]
fn mem_allocate_meta() {
    let mut mem = Memory::new();
    
    {
        let loc1 = Location::Stdin { line: 23, column: 42 };
        let md1  = Metadata{ read_name: "".to_string(), location: loc1, documentation: "".to_string(), parameters: vec![] };
        let x1   = mem.allocate_number(1370).with_metadata(md1);

        let loc2 = Location::File { path: PathBuf::from("~/the/input/file.lisp"), line: 41, column: 42 };
        let md2  = Metadata{ read_name: "".to_string(), location: loc2, documentation: "Very important information".to_string(), parameters: vec![] };
        let x2   = mem.allocate_character(' ').with_metadata(md2);

        let y    = mem.symbol_for("thing");

        assert_eq!(*x1.get().unwrap().as_number(),    1370);
        assert_eq!(*x2.get().unwrap().as_character(), ' ');

        assert_eq!(x1.get_metadata().unwrap().location.get_file(), None);
        assert_eq!(x1.get_metadata().unwrap().location.get_line().unwrap(), 23);
        assert_eq!(x1.get_metadata().unwrap().documentation, "");
        assert_eq!(x2.get_metadata().unwrap().location.get_file().unwrap(), PathBuf::from("~/the/input/file.lisp"));
        assert_eq!(x2.get_metadata().unwrap().location.get_column().unwrap(), 42);
        assert_eq!(x2.get_metadata().unwrap().documentation, "Very important information");

        assert_eq!(y.get_metadata(), None);
    }

    mem.collect();

    assert_eq!(mem.used_count(), 0);
}

#[test]
fn mem_globals() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(140);
    mem.define_global("x", x);
    let y = mem.symbol_for("thing");
    mem.define_global("y", y);

    assert_eq!(*mem.get_global("x").unwrap().get().unwrap().as_number(), 140);
    assert!(mem.get_global("z").is_none());

    mem.undefine_global("x");
    assert!(mem.get_global("x").is_none());

    mem.collect();

    assert_eq!(mem.get_global("y").unwrap().get().unwrap().as_symbol(), mem.symbol_for("thing").get().unwrap().as_symbol());
}

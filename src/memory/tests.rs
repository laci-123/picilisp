use pretty_assertions::assert_eq;
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

    mem.collect();

    assert_eq!(mem.used_cells.len(), 0);
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


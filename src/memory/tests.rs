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
            refs.push(mem.allocate(PrimitiveValue::Number(i as f64)));
        }

        assert_eq!(mem.free_cells.len(), 0);
        assert_eq!(mem.used_cells.len(), INITIAL_FREE_CELLS);

        refs.push(mem.allocate(PrimitiveValue::Number(-1.0)));

        assert_eq!(mem.free_cells.len(), ALLOCATION_INCREMENT - 1);
        assert_eq!(mem.used_cells.len(), INITIAL_FREE_CELLS + 1);

        for i in 0 .. 100 {
            refs.push(mem.allocate(PrimitiveValue::Number(i as f64)));
        }

        // drop refs
    }

    // make sure at least one garbage collection happens
    for i in 100 .. 200 {
        mem.allocate(PrimitiveValue::Number(i as f64));
    }

    assert_eq!(mem.used_cells.len(), 1);
    assert_eq!(mem.free_cells.len(), ALLOCATION_INCREMENT - 1);
}


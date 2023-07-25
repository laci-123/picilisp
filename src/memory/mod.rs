#![allow(dead_code)]

use std::collections::HashSet;


#[derive(Default)]
pub struct Cell {
    content: Box<CellContent>,
}

impl Cell {
    pub fn new(value: PrimitiveValue) -> Self {
        Self{ content: Box::new(CellContent::new(value)) }
    }
    
    pub fn set(&mut self, value: PrimitiveValue) {
        self.content.as_mut().value = value;
    }
    
    pub fn as_ptr_mut(&self) -> *mut CellContent {
        (&*self.content as *const CellContent) as *mut CellContent
    }
}


#[derive(Default)]
pub struct CellContent {
    value: PrimitiveValue,
    external_ref_count: usize,
}

impl CellContent {
    pub fn new(value: PrimitiveValue) -> Self {
        Self{ value, external_ref_count: 0 }
    }
}


pub struct ConsCell {
    car: *mut CellContent,
    cdr: *mut CellContent,
}


#[derive(Default)]
pub enum PrimitiveValue {
    #[default]
    Nil,
    Number(f64),
    Character(char),
    Cons(ConsCell),
    // TODO: Symbol, Function, SignalHandler
}

impl PrimitiveValue {
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn as_number(&self) -> &f64 {
        if let Self::Number(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-number PrimitiveValue to number")
        }
    }

    pub fn as_character(&self) -> &char {
        if let Self::Character(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-character PrimitiveValue to character")
        }
    }

    pub fn as_conscell(&self) -> &ConsCell {
        if let Self::Cons(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-conscell PrimitiveValue to conscell")
        }
    }
}


pub struct ExternalRefrence {
    pointer: *mut CellContent,
}

impl ExternalRefrence {
    fn new(pointer: *mut CellContent) -> Self {
        unsafe {
            (*pointer).external_ref_count += 1;
        }

        Self{ pointer }
    }
}

impl Drop for ExternalRefrence {
    fn drop(&mut self) {
        unsafe {
            (*self.pointer).external_ref_count -= 1;
        }
    }
}


pub struct Memory {
    used_cells: Vec<Cell>,
    free_cells: Vec<Cell>,
}

/// number of free cells when the [Memory] is constructed
const INITIAL_FREE_CELLS: usize = 8;
/// maximum ratio of the number of free cells after garbage collection, compared to the number of used cells
const MAXIMUM_FREE_RATIO: f32   = 0.75;
/// when removing free cells after garbage collection, keep as many that the ratio of their number and the number
/// of used cells is at least this big
const MINIMUM_FREE_RATIO: f32   = 0.1;
/// when there are no more free cells (not even after garbage collection), allocate this many
const ALLOCATION_INCREMENT: usize = 8;

impl Memory {
    pub fn new() -> Self {
        Self { used_cells: vec![],
               free_cells: (0 .. INITIAL_FREE_CELLS).map(|_| Default::default()).collect() }
    }

    pub fn allocate(&mut self, value: PrimitiveValue) -> ExternalRefrence {
        let ptr = self.allocate_internal(value);
        ExternalRefrence::new(ptr)
    }

    fn allocate_internal(&mut self, value: PrimitiveValue) -> *mut CellContent {
        if self.free_cells.len() == 0 {
            self.collect();
        }
        
        if let Some(mut cell) = self.free_cells.pop() {
            cell.set(value);
            let ptr = cell.as_ptr_mut();
            self.used_cells.push(cell);
            ptr
        }
        else {
            for _ in 1 .. ALLOCATION_INCREMENT {
                // pre-allocate a bunch of cells
                // so that `collect` won't have to run
                // on the next few times `allocate_internal` is called
                self.free_cells.push(Default::default());
            }

            let new_cell = Cell::new(value);
            let ptr = new_cell.as_ptr_mut();
            self.used_cells.push(new_cell);
            ptr
        }
    }

    fn collect(&mut self) {
        // find cells that are externally reachable (the roots)
        let mut stack = vec![];

        for cell in self.used_cells.iter() {
            if cell.content.external_ref_count > 0 {
                stack.push(cell.as_ptr_mut());
            }
        }

        // find all reachable cells
        // starting from the roots
        // (DFS)
        let mut reachable = HashSet::new();

        while let Some(cell) = stack.pop() {
            reachable.insert(cell);

            let value = unsafe{ &(*cell).value };
            match value {
                PrimitiveValue::Cons(cons) => {
                    stack.push(cons.car);
                    stack.push(cons.cdr);
                },
                _ =>{},
            }
        }

        // remove unreachable cells
        let mut i = 0;
        while i < self.used_cells.len() {
            let ptr = self.used_cells[i].as_ptr_mut();
            if reachable.contains(&ptr) {
                self.free_cells.push(self.used_cells.swap_remove(i));
            }
            else {
                i += 1;
            }
        }

        // if there are too many free cells
        // then remove some, but not too many
        let max_free_cells = (self.used_cells.len() as f32 * MAXIMUM_FREE_RATIO) as usize;

        if self.free_cells.len() > max_free_cells {
            let min_free_cells = (self.used_cells.len() as f32 * MINIMUM_FREE_RATIO) as usize;
            self.free_cells.truncate(min_free_cells);
        }
    }
}

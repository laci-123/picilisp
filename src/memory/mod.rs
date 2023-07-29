#![allow(dead_code)]

use std::collections::{HashSet, HashMap};


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


pub struct Symbol {
    name: Option<String>,
    own_address: *const CellContent,
    // TODO: position (file, line, column)
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        if self.own_address.is_null() {
            false
        }
        else {
            self.own_address == other.own_address
        }
    }
}


#[derive(Default)]
pub enum PrimitiveValue {
    #[default]
    Nil,
    Number(f64),
    Character(char),
    Cons(ConsCell),
    Symbol(Symbol),
    // TODO: Function, SignalHandler
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

    pub fn as_symbol(&self) -> &Symbol{
        if let Self::Symbol(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-symbol PrimitiveValue to symbol")
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
    symbols: HashMap<String, *const CellContent>,
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
               free_cells: (0 .. INITIAL_FREE_CELLS).map(|_| Default::default()).collect(),
               symbols:    HashMap::new() }
    }

    pub fn symbol_for(&mut self, name: &str) -> ExternalRefrence {
        if let Some(sym_ptr) = self.symbols.get(name) {
            ExternalRefrence::new(*sym_ptr as *mut CellContent)
        }
        else {
            let sym_ptr = self.allocate_internal(PrimitiveValue::Symbol(Symbol{ name: Some(name.to_string()), own_address: std::ptr::null() }));
            if let PrimitiveValue::Symbol(sym) = unsafe { &mut (*sym_ptr).value } {
                sym.own_address = sym_ptr;
            }
            else {
                unreachable!();
            }

            self.symbols.insert(name.to_string(), sym_ptr);

            ExternalRefrence::new(sym_ptr)
        }
    }

    pub fn unique_symbol(&mut self) -> ExternalRefrence {
        let sym_ptr = self.allocate_internal(PrimitiveValue::Symbol(Symbol{ name: None, own_address: std::ptr::null() }));
        ExternalRefrence::new(sym_ptr)
    }

    pub fn allocate_number(&mut self, number: f64) -> ExternalRefrence {
        let ptr = self.allocate_internal(PrimitiveValue::Number(number));
        ExternalRefrence::new(ptr)
    }

    pub fn allocate_character(&mut self, character: char) -> ExternalRefrence {
        let ptr = self.allocate_internal(PrimitiveValue::Character(character));
        ExternalRefrence::new(ptr)
    }

    pub fn allocate_cons(&mut self, car: Option<ExternalRefrence>, cdr: Option<ExternalRefrence>) -> ExternalRefrence {
        let ptr = self.allocate_internal(PrimitiveValue::Cons(ConsCell{ car: car.map_or(std::ptr::null_mut(), |c| c.pointer),
                                                                        cdr: cdr.map_or(std::ptr::null_mut(), |c| c.pointer) }));
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
                i += 1;
            }
            else {
                self.free_cells.push(self.used_cells.swap_remove(i));
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

    fn dump_memory(&self) {
        let used_count = self.used_cells.len();
        let free_count = self.free_cells.len();
        let total_count = used_count + free_count;
        let total_size_kb = (total_count * (std::mem::size_of::<Cell>() + std::mem::size_of::<CellContent>())) as f32 / 1024.0;
        
        println!("Total: {} cells ({:.2} kB)", total_count, total_size_kb);
        println!("  - used: {}", used_count);
        println!("  - free: {}", free_count);
        println!("");

        println!("Used: ");
        println!("Address        Type      Value                                    External RefCount");
        println!("-------        ----      -----                                    -----------------");
        for c in self.used_cells.iter() {
            let string = 
            match c.content.value {
                PrimitiveValue::Nil            => format!("NIL       NIL"),
                PrimitiveValue::Number(n)      => format!("NUMBER    {n}"),
                PrimitiveValue::Character(ch)  => format!("CHARACTER {ch}"),
                PrimitiveValue::Symbol(ref s)  => format!("{}", s.name.as_ref().map_or("UNIQUE SYMBOL".to_string(), |n| format!("SYMBOL \"{n}\""))),
                PrimitiveValue::Cons(ref cons) => format!("CONS      car: {car:p} cdr: {cdr:p}", car = cons.car, cdr = cons.cdr),
            };
            let rc = c.content.external_ref_count;
            println!("{:p} {:<50} {}", c.as_ptr(), string, rc);
        }
        println!("");

        println!("Free: ");
        println!("Address        Type      Value                                    External RefCount");
        println!("_______        ____      _____                                    _________________");
        for c in self.free_cells.iter() {
            let string = 
            match c.content.value {
                PrimitiveValue::Nil            => format!("NIL       NIL"),
                PrimitiveValue::Number(n)      => format!("NUMBER    {n}"),
                PrimitiveValue::Character(ch)  => format!("CHARACTER {ch}"),
                PrimitiveValue::Symbol(ref s)  => format!("{}", s.name.as_ref().map_or("UNIQUE SYMBOL".to_string(), |n| format!("SYMBOL \"{n}\""))),
                PrimitiveValue::Cons(ref cons) => format!("CONS      car: {car:p} cdr: {cdr:p}", car = cons.car, cdr = cons.cdr),
            };
            let rc = c.content.external_ref_count;
            println!("{:p} {:<50} {}", c.as_ptr(), string, rc);
        }

        println!("");
    }
}


#[cfg(test)]
mod tests;

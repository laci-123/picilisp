#![allow(dead_code)]



struct Cell {
    content: Box<CellContent>,
}

impl Cell {
    pub fn as_ptr_mut(&self) -> *mut CellContent {
        (&*self.content as *const CellContent) as *mut CellContent
    }
}


struct CellContent {
    value: PrimitiveValue,
    external_ref_count: usize,
}


pub struct ConsCell {
    car: *mut CellContent,
    cdr: *mut CellContent,
}


pub enum PrimitiveValue {
    Nil,
    Number(f32),
    Character(char),
    Cons(ConsCell),
    // TODO: Symbol, Function, SignalHandler
}

impl PrimitiveValue {
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn as_number(&self) -> &f32 {
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

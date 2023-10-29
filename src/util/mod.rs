use crate::memory::*;



pub struct StringIterator {
    input: GcRef,
}

impl StringIterator {
    pub fn new(input: GcRef) -> Self {
        Self{ input }
    }
}

impl Iterator for StringIterator {
    type Item = Option<(char, GcRef)>; // Some(None): input is not a valid string

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_nil() {
            return None;
        }
        
        if let Some(PrimitiveValue::Cons(cons)) = self.input.get() {
            if let Some(PrimitiveValue::Character(c)) = cons.get_car().get() {
                let rest = cons.get_cdr();
                self.input = cons.get_cdr();
                Some(Some((*c, rest)))
        }
            else {
                Some(None)
            }
        }
        else {
            Some(None)
        }
    }
}


/// Converts a vector of primitive values to a Lisp-style list of primitive values
///
/// For example: [1, 2, 3] -> (cons 1 (cons 2 (cons 3 nil)))
pub fn vec_to_list(mem: &mut Memory, vec: &[GcRef]) -> GcRef {
    let mut c = GcRef::nil();

    for v in vec.iter().rev() {
        c = mem.allocate_cons(v.clone(), c);
    }

    c
}


pub struct ConsType {
    pub is_list: bool,
    pub is_string: bool,
}


pub fn cons_type(thing: GcRef) -> ConsType {
    let mut is_string = true;
    let mut cursor = thing;
    loop {
        if cursor.is_nil() {
            break;
        }

        if let Some(PrimitiveValue::Cons(cons)) = cursor.get() {
            if cons.get_car().get_type() != TypeLabel::Character {
                is_string = false;
            }
            cursor = cons.get_cdr();
        }
        else {
            return ConsType{ is_string: false, is_list: false };
        }
    }
    
    ConsType{ is_string, is_list: true }
}


pub fn string_to_list(mem: &mut Memory, string: &str) -> GcRef {
    let char_vec = string.chars().map(|c| mem.allocate_character(c)).collect::<Vec<GcRef>>();
    vec_to_list(mem, &char_vec)
}


pub fn string_to_proper_list(mem: &mut Memory, string: &str) -> GcRef {
    let mut char_vec = string.chars().map(|c| mem.allocate_character(c)).collect::<Vec<GcRef>>();
    char_vec.insert(0, mem.symbol_for("list"));
    vec_to_list(mem, &char_vec)
}


pub fn list_to_string(list: GcRef) -> Option<String> {
    if let Some(vec) = list_to_vec(list) {
        let mut string = String::new();

        let mut from = 0;
        if let Some(x) = vec.first() {
            if let Some(PrimitiveValue::Symbol(sym)) = x.get() {
                if sym.get_name() == "list" {
                    from = 1;
                }
            }
        }

        for x in &vec[from..] {
            if let Some(PrimitiveValue::Character(c)) = x.get() {
                string.push(*c);
            }
            else {
                return None;
            }
        }

        Some(string)
    }
    else {
        None
    }
}


/// Converts a Lisp-style list of primitive values to a vector of primitive values
///
/// Returns None if `list` is not a valid list, i.e. if it is not a cons cell (or `nil` in case of the empty list)
/// or if the `cdr` of its last cons cell is not `nil`.
/// 
/// For example:  (cons 1 (cons 2 (cons 3 nil))) -> [1, 2, 3]
pub fn list_to_vec(list: GcRef) -> Option<Vec<GcRef>> {
    let mut vec = vec![];
    
    let mut cursor = list;
    loop {
        if cursor.is_nil() {
            break;
        }

        if let Some(PrimitiveValue::Cons(cons)) = cursor.get() {
            vec.push(cons.get_car());
            cursor = cons.get_cdr();
        }
        else {
            return None;
        }
    }

    Some(vec)
}


pub fn append_lists(mem: &mut Memory, list1: GcRef, list2: GcRef) -> Option<GcRef> {
    let mut c = list2;
    let vec1 = list_to_vec(list1)?;
    for x in vec1.iter().rev() {
        c = mem.allocate_cons(x.clone(), c);
    }
    Some(c)
}


pub struct Defer<F: Fn() -> ()> {
    f: F,
}

impl<F: Fn() -> ()> Defer<F> {
    pub fn new(f: F) -> Self {
        Self{ f }
    }
}

impl<F: Fn() -> ()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.f)();
    }
}


macro_rules! symbol_eq {
    ($x:expr, $y:expr) => {
        if let Some(PrimitiveValue::Symbol(s1)) = $x.get() {
            if let Some(PrimitiveValue::Symbol(s2)) = $y.get() {
                s1 == s2
            }
            else {
                false
            }
        }
        else {
            false
        }
    };
}

pub(crate) use symbol_eq;


#[cfg(test)]
macro_rules! assert_eq_symbol {
    ($x:expr, $y:expr) => {
        assert_eq!($x.get().unwrap().as_symbol(), $y.get().unwrap().as_symbol())
    };
}

#[cfg(test)]
pub(crate) use assert_eq_symbol;



#[cfg(test)]
mod tests;

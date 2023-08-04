#![allow(dead_code)]

use crate::memory::*;
// use std::collections::VecDeque;


/// Converts a vector of primitive values to a Lisp-style list of primitive values
///
/// For example: [1, 2, 3] -> (cons 1 (cons 2 (cons 3 nil)))
pub fn vec_to_list(mem: &mut Memory, mut vec: Vec<ExternalReference>) -> ExternalReference {
    vec.reverse();
    vec_to_list_reverse(mem, vec)
}


/// Converts a vector of primitive values to a Lisp-style list of primitive values, in reverse order
///
/// For example: [3, 2, 1] -> (cons 1 (cons 2 (cons 3 nil)))
pub fn vec_to_list_reverse(mem: &mut Memory, vec: Vec<ExternalReference>) -> ExternalReference {
    let mut c = ExternalReference::nil();

    for v in vec {
        c = mem.allocate_cons(v, c);
    }

    c
}


pub fn string_to_list(mem: &mut Memory, string: &str) -> ExternalReference {
    let char_vec = string.chars().map(|c| mem.allocate_character(c)).collect();
    vec_to_list(mem, char_vec)
}


pub fn list_to_string(list: ExternalReference) -> Option<String> {
    list_to_vec(list).map(|vec| vec.iter().map(|x| x.get().as_character()).collect())
}


/// Converts a Lisp-style list of primitive values to a vector of primitive values
///
/// Returns None if `list` is not a valid list, i.e. if it is not a cons cell (or `nil` in case of the empty list)
/// or if the `cdr` of its last cons cell is not `nil`.
/// 
/// For example:  (cons 1 (cons 2 (cons 3 nil))) -> [1, 2, 3]
pub fn list_to_vec(list: ExternalReference) -> Option<Vec<ExternalReference>> {
    let mut vec = vec![];
    
    let mut cursor = list;
    loop {
        if cursor.is_nil() {
            break;
        }

        if let PrimitiveValue::Cons(cons) = cursor.get() {
            vec.push(cons.get_car());
            cursor = cons.get_cdr();
        }
        else {
            return None;
        }
    }

    Some(vec)
}


pub fn append_lists(mem: &mut Memory, list1: ExternalReference, list2: ExternalReference) -> Option<ExternalReference> {
    let mut c = list2;
    let vec1 = list_to_vec(list1)?;
    for x in vec1.iter().rev() {
        c = mem.allocate_cons(x.clone(), c);
    }
    Some(c)
}


pub enum FoldOutput {
    Return(ExternalReference),
    Call(ExternalReference, ExternalReference),
    Signal(ExternalReference),
}

impl FoldOutput {
    pub fn as_value(self) -> ExternalReference {
        if let Self::Return(x) = self {
            x
        }
        else {
            panic!("attempted to get return value from a non-Return FoldOutput")
        }
    }

    pub fn as_signal(self) -> ExternalReference {
        if let Self::Signal(x) = self {
            x
        }
        else {
            panic!("attempted to get signal from a non-Signal FoldOutput")
        }
    }
}

#[cfg(test)]
mod tests;

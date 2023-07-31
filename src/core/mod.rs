#![allow(dead_code)]

use crate::memory::*;


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


#[cfg(test)]
mod tests;

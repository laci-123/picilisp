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

    vec.reverse();
    Some(vec)
}


// pub enum FoldInput {
//     Singleton(ExternalReference),
//     List(Vec<ExternalReference>),
// }


// pub enum FoldOutput {
//     Return(ExternalReference),
//     Signal(ExternalReference),
// }


// pub fn fold_tree<T>(mem: &mut Memory, tree: ExternalReference, initial: T, f: impl Fn(T, FoldInput) -> FoldOutput) -> T {
//     todo!()
// }


#[cfg(test)]
mod tests;

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


struct Atom {
    value: ExternalReference,
    in_call: bool,
    state: ExternalReference,
}

struct List {
    elems: Vec<ExternalReference>,
    current: usize,
    in_call: bool,
    state: ExternalReference,
}

enum StackFrame {
    Atom(Atom),
    List(List),
}

impl StackFrame {
    fn new(x: ExternalReference, initial_state: ExternalReference) -> Self {
        if let Some(vec) = list_to_vec(x.clone()) {
            Self::List(List{ elems: vec, current: 0, in_call: false, state: initial_state })
        }
        else {
            Self::Atom(Atom{ value: x, in_call: false, state: initial_state })
        }
    }
}


pub fn fold_tree(mem:    &mut Memory,
                 state:  ExternalReference,
                 tree:   ExternalReference,
                 f_atom: impl Fn(&mut Memory, ExternalReference, ExternalReference) -> ExternalReference,
                 f_list: impl Fn(&mut Memory, ExternalReference, Vec<ExternalReference>) -> FoldOutput)
                 -> FoldOutput
{
    let mut stack = vec![StackFrame::new(tree, state)];
    let mut return_value = ExternalReference::nil();

    'stack_loop: while let Some(frame) = stack.last_mut() {
        match frame {
            StackFrame::Atom(atom_frame) => {
                if atom_frame.in_call {
                    // Don't need to do anything.
                    // The next frame (that evaluated trap.normal_body) set return_value,
                    // we just leave it as it is.
                }
                else {
                    if let PrimitiveValue::Trap(trap) = atom_frame.value.get() {
                        let new_tree       = trap.get_normal_body();
                        let new_state      = atom_frame.state.clone();
                        atom_frame.in_call = true;
                        stack.push(StackFrame::new(new_tree, new_state));
                        continue 'stack_loop;
                    }
                    return_value = f_atom(mem, atom_frame.state.clone(), atom_frame.value.clone());
                }
            },
            StackFrame::List(list_frame) => {
                if list_frame.in_call {
                    list_frame.elems[list_frame.current] = return_value.clone();
                    list_frame.current += 1;
                    list_frame.in_call = false;
                }

                for i in list_frame.current .. list_frame.elems.len() {
                    let x = list_frame.elems[i].clone();
                    list_frame.in_call = true;
                    let s = list_frame.state.clone();
                    stack.push(StackFrame::new(x, s));
                    continue 'stack_loop;
                }

                match f_list(mem, list_frame.state.clone(), list_frame.elems.clone()) {
                    FoldOutput::Return(x) => {
                        return_value = x;
                    },
                    FoldOutput::Call(new_tree, new_state) => {
                        *frame = StackFrame::new(new_tree, new_state);
                        continue 'stack_loop;
                    },
                    FoldOutput::Signal(signal) => {
                        while let Some(old_frame) = stack.pop() {
                            if let StackFrame::Atom(af) = old_frame {
                                if let PrimitiveValue::Trap(trap) = af.value.get() {
                                    // TODO: give `signal` to `trap_body` somehow
                                    stack.push(StackFrame::new(trap.get_trap_body(), af.state));
                                    continue 'stack_loop;
                                }
                            }
                        }

                        return FoldOutput::Signal(signal);
                    },
                }
            },
        }

        stack.pop();
    }

    FoldOutput::Return(return_value)
}


#[cfg(test)]
mod tests;

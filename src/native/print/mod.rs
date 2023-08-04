use crate::core::*;
use crate::memory::*;


fn print_atom(mem: &mut Memory, atom: ExternalReference) -> ExternalReference {
    match atom.get() {
        PrimitiveValue::Nil          => string_to_list(mem, "()"),
        PrimitiveValue::Number(x)    => string_to_list(mem, &format!("{x}")),
        PrimitiveValue::Character(x) => string_to_list(mem, &format!("%{x}")),
        PrimitiveValue::Symbol(x)    => string_to_list(mem, &format!("{}", x.get_name())),
        PrimitiveValue::Function(_)  => string_to_list(mem, &format!("#<function>")),
        PrimitiveValue::Trap(_)      => string_to_list(mem, &format!("#<trap>")),
        PrimitiveValue::Meta(x)      => print_atom(mem, x.get_value()),
        PrimitiveValue::Cons(x)      => {
            let car = list_to_string(print_atom(mem, x.get_car())).unwrap();
            let cdr = list_to_string(print_atom(mem, x.get_cdr())).unwrap();
            string_to_list(mem, &format!("(cons {car} {cdr})"))
        },
    }
}

fn print_string(mem: &mut Memory, list: Vec<ExternalReference>) -> ExternalReference {
    let mut result = string_to_list(mem, "\"");
    for x in list.iter().rev() {
        result = mem.allocate_cons(x.clone(), result);
    }

    let quote = mem.allocate_character('"');
    mem.allocate_cons(quote, result)
}

fn print_list(mem: &mut Memory, list: Vec<ExternalReference>) -> ExternalReference {
    let mut result = string_to_list(mem, ")");
    let space = string_to_list(mem, " ");
    for (i, x) in list.iter().rev().enumerate() {
        result = append_lists(mem, x.clone(), result).unwrap();
        if i < list.len() - 1 {
            result = append_lists(mem, space.clone(), result).unwrap();
        }
    }

    let open_paren = string_to_list(mem, "(");
    append_lists(mem, open_paren, result).unwrap()
}

struct Atom {
    value: ExternalReference,
}

struct List {
    elems: Vec<ExternalReference>,
    current: usize,
    in_call: bool,
}

enum StackFrame {
    Atom(Atom),
    List(List),
}

impl StackFrame {
    fn new(x: ExternalReference) -> Self {
        if let Some(vec) = list_to_vec(x.clone()) {
            Self::List(List{ elems: vec, current: 0, in_call: false })
        }
        else {
            Self::Atom(Atom{ value: x })
        }
    }
}

pub fn print(mem: &mut Memory, tree: ExternalReference) -> ExternalReference
{
    let mut stack        = vec![StackFrame::new(tree)];
    let mut return_value = ExternalReference::nil();

    'stack_loop: while let Some(frame) = stack.last_mut() {
        match frame {
            StackFrame::Atom(atom_frame) => {
                return_value = print_atom(mem, atom_frame.value.clone());
            },
            StackFrame::List(list_frame) => {
                if list_frame.in_call {
                    list_frame.elems[list_frame.current] = return_value.clone();
                    list_frame.current += 1;
                    list_frame.in_call = false;
                }

                if list_frame.elems.len() > 0 && list_frame.elems.iter().all(|x| matches!(x.get(), PrimitiveValue::Character(_))) {
                    return_value = print_string(mem, list_frame.elems.clone());
                }
                else {
                    for i in list_frame.current .. list_frame.elems.len() {
                        let x = list_frame.elems[i].clone();
                        list_frame.in_call = true;
                        stack.push(StackFrame::new(x));
                        continue 'stack_loop;
                    }

                    return_value = print_list(mem, list_frame.elems.clone());
                }
            },
        }

        stack.pop();
    }

    return_value
}

#[cfg(test)]
mod tests;

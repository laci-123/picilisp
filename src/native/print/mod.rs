use crate::core::*;
use crate::memory::*;


fn print_atom(mem: &mut Memory, _state: ExternalReference, atom: ExternalReference) -> ExternalReference {
    match atom.get() {
        PrimitiveValue::Nil          => string_to_list(mem, "()"),
        PrimitiveValue::Number(x)    => string_to_list(mem, &format!("{x}")),
        PrimitiveValue::Character(x) => string_to_list(mem, &format!("%{x}")),
        PrimitiveValue::Symbol(x)    => string_to_list(mem, &format!("{}", x.get_name())),
        PrimitiveValue::Function(_)  => string_to_list(mem, &format!("#<function>")),
        PrimitiveValue::Trap(_)      => string_to_list(mem, &format!("#<trap>")),
        PrimitiveValue::Meta(x)      => print_atom(mem, ExternalReference::nil(), x.get_value()),
        PrimitiveValue::Cons(x)      => {
            let car = list_to_string(print_atom(mem, ExternalReference::nil(), x.get_car())).unwrap();
            let cdr = list_to_string(print_atom(mem, ExternalReference::nil(), x.get_cdr())).unwrap();
            string_to_list(mem, &format!("(cons {car} {cdr})"))
        },
    }
}

fn print_list(mem: &mut Memory, _state: ExternalReference, list: Vec<ExternalReference>) -> FoldOutput {
    let mut cursor = string_to_list(mem, ")");
    let space = string_to_list(mem, " ");
    for (i, x) in list.iter().rev().enumerate() {
        cursor = append_lists(mem, x.clone(), cursor).unwrap();
        if i < list.len() - 1 {
            cursor = append_lists(mem, space.clone(), cursor).unwrap();
        }
    }

    let open_paren = string_to_list(mem, "(");
    FoldOutput::Return(append_lists(mem, open_paren, cursor).unwrap())
}

pub fn print(mem: &mut Memory, value: ExternalReference) -> ExternalReference {
    fold_tree(mem, ExternalReference::nil(), value, print_atom, print_list).as_value()
}


#[cfg(test)]
mod tests;

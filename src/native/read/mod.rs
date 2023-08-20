#![allow(dead_code)]

use crate::memory::*;
use crate::util::{vec_to_list, string_to_list};


/// Converts a Lisp-style string to an AST
///
/// Only reads the shortest prefix of `input` that is a valid AST
///
/// Returns: `(list status result rest)`
/// where `status` can be one of the following:
///  * `ok`:         Success. `result` is the AST.
///  * `incomplete`: `input` is not a valid AST, but can be the beginning of a valid AST. `result` is undefined.
///  * `error`:      `input` is not a valid AST, not even the beginning of one. `result` contains the error details.
///  * `invalid`:    `input` is not a valid string. `result` and `rest` are is undefined.
/// `result` is the read AST and
/// `rest` is the unread rest of `input`.
fn read_internal(mem: &mut Memory, input: GcRef) -> GcRef {
    use State::*;
    
    let ok_sym          = mem.symbol_for("ok");
    let incomplete_sym  = mem.symbol_for("incomplete");
    let invalid_sym     = mem.symbol_for("invalid");
    let error_sym       = mem.symbol_for("error");
    let invalid         = vec_to_list(mem, vec![invalid_sym, GcRef::nil(), GcRef::nil(), GcRef::nil()]);

    let mut atom_stack  = vec![];
    let mut list_stack  = vec![];
    let mut state       = WhiteSpace;
    let mut cursor      = input;
    let mut next_cursor;
    let mut ch;

    while !cursor.is_nil() {
        if let Some((head, tail)) = fetch_character(cursor.clone()) {
            ch = head;
            next_cursor = tail;
        }
        else {
            return invalid;
        }

        match (state, ch) {
            (WhiteSpace, ';') => {
                state = Comment;
            },
            (WhiteSpace, '(') => {
                list_stack.push(ListStack::Separator);
            },
            (WhiteSpace, '"') => {
                list_stack.push(ListStack::Separator);
                state = StringNormal;
            },
            (WhiteSpace, c) if c.is_whitespace() => {
                // do nothing
            },
            (WhiteSpace, ')')  => { 
                if let Err(error_msg) = build_list(mem, &mut list_stack) {
                    return vec_to_list(mem, vec![error_sym, error_msg, cursor]);
                }
            },
            (WhiteSpace, c) => {
                atom_stack.push(c);
                state = Atom;
            },
            (Comment, '\n') => {
                state = WhiteSpace
            },
            (Comment, _) => {
                // do nothing
            },
            (Atom, ')') => {
                list_stack.push(ListStack::Elem(read_atom(mem, &atom_stack.drain(..).collect::<String>())));

                if let Err(error_msg) = build_list(mem, &mut list_stack) {
                    return vec_to_list(mem, vec![error_sym, error_msg, cursor]);
                }

                state = WhiteSpace;
            },
            (Atom, '(') => {
                let atom = read_atom(mem, &atom_stack.drain(..).collect::<String>());

                if list_stack.len() > 0 {
                    list_stack.push(ListStack::Elem(atom));
                }
                else {
                    return vec_to_list(mem, vec![ok_sym, atom, cursor]);
                }

                list_stack.push(ListStack::Separator);

                state = WhiteSpace;
            },
            (Atom, '"') => {
                let atom = read_atom(mem, &atom_stack.drain(..).collect::<String>());

                if list_stack.len() > 0 {
                    list_stack.push(ListStack::Elem(atom));
                }
                else {
                    return vec_to_list(mem, vec![ok_sym, atom, cursor]);
                }

                list_stack.push(ListStack::Separator);

                state = StringNormal;
            },
            (Atom, ';') => {
                let atom = read_atom(mem, &atom_stack.drain(..).collect::<String>());

                if list_stack.len() > 0 {
                    list_stack.push(ListStack::Elem(atom));
                }
                else {
                    return vec_to_list(mem, vec![ok_sym, atom, cursor]);
                }

                state = Comment;
            },
            (Atom, c) if c.is_whitespace() => {
                let atom = read_atom(mem, &atom_stack.drain(..).collect::<String>());

                if list_stack.len() > 0 {
                    list_stack.push(ListStack::Elem(atom));
                }
                else {
                    return vec_to_list(mem, vec![ok_sym, atom, cursor]);
                }

                state = WhiteSpace;
            },
            (Atom, c) => {
                atom_stack.push(c);
            },
            (StringNormal, '"') => {
                if let Err(error_msg) = build_list(mem, &mut list_stack) {
                    return vec_to_list(mem, vec![error_sym, error_msg, cursor]);
                }

                state = WhiteSpace;
            },
            (StringNormal, '\\') => {
                state = StringEscape;
            },
            (StringNormal, c) => {
                list_stack.push(ListStack::Elem(mem.allocate_character(c)));
            },
            (StringEscape, c) => {
                let escaped =
                match c {
                    'n'   => '\n',
                    't'   => '\t',
                    'r'   => '\r',
                    '"'   => '"',
                    '\\'  => '\\',
                    other => {
                        let error_msg = string_to_list(mem, &format!("'{other}' is not a valid escape character in a string literal"));
                        return vec_to_list(mem, vec![error_sym, error_msg, cursor]);
                    },
                };
                list_stack.push(ListStack::Elem(mem.allocate_character(escaped)));
                state = StringNormal;
            },
        }

        cursor = next_cursor;
    }

    if let Some(first) = list_stack.first() {
        if let ListStack::Elem(elem) = first {
            vec_to_list(mem, vec![ok_sym, elem.clone(), cursor])
        }
        else {
            vec_to_list(mem, vec![incomplete_sym, GcRef::nil(), cursor])
        }
    }
    else {
        if atom_stack.len() > 0 {
            let result = read_atom(mem, &atom_stack.drain(..).collect::<String>());
            vec_to_list(mem, vec![ok_sym, result, cursor])
        }
        else {
            vec_to_list(mem, vec![incomplete_sym, GcRef::nil(), cursor])
        }
    }
}


enum ListStack {
    Separator,
    Elem(GcRef),
}


#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    WhiteSpace,
    Comment,
    StringNormal,
    StringEscape,
    Atom,
}


fn fetch_character(input: GcRef) -> Option<(char, GcRef)> {
    let cons =
    if let PrimitiveValue::Cons(cons) = input.get() {
        cons
    }
    else {
        return None;
    };

    if let PrimitiveValue::Character(ch) = cons.get_car().get() {
        Some((*ch, cons.get_cdr()))
    }
    else {
        None
    }
}

fn build_list(mem: &mut Memory, list_stack: &mut Vec<ListStack>) -> Result<(), GcRef> {
    if list_stack.len() == 0 || (list_stack.len() == 1 && !matches!(list_stack[0], ListStack::Separator)) {
        let error_msg = string_to_list(mem, "too many closing parentheses");
        return Err(error_msg);
    }

    let mut new_list = GcRef::nil();
    while let Some(x) = list_stack.pop() {
        if let ListStack::Elem(elem) = x {
            new_list = mem.allocate_cons(elem, new_list);
        }
        else {
            list_stack.push(ListStack::Elem(new_list));
            break;
        }
    }

    Ok(())
}


fn read_atom(mem: &mut Memory, string: &str) -> GcRef {
    if let Some(x) = read_number(mem, string) {
        return x;
    }
    if let Some(x) = read_character(mem, string) {
        return x;
    }
    read_symbol(mem, string)
}


fn read_number(mem: &mut Memory, string: &str) -> Option<GcRef> {
    if let Ok(x) = string.parse() {
        Some(mem.allocate_number(x))
    }
    else {
        None
    }
}

fn read_character(mem: &mut Memory, string: &str) -> Option<GcRef> {
    let mut chars = string.chars();
    if let Some('%') = chars.next() {
        let c1 = chars.next()?;
        if c1 == '\\' {
            let c2 = chars.next()?;
            if chars.next().is_none() {
                let c3 =
                match c2 {
                    't' => '\t',
                    'n' => '\n',
                    'r' => '\r',
                    '\\' => '\\',
                    _    => return None, 
                };
                Some(mem.allocate_character(c3))
            }
            else {
                None
            }
        }
        else {
            Some(mem.allocate_character(c1))
        }
    }
    else {
        None
    }
}

fn read_symbol(mem: &mut Memory, string: &str) -> GcRef {
    mem.symbol_for(&string)
}


pub fn read(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    NativeResult::Value(read_internal(mem, args[0].clone()))
}


#[cfg(test)]
mod tests;

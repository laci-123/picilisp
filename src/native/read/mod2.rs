use crate::memory::*;


/// Converts a Lisp-style string to an AST
///
/// Only reads the shortest prefix of `input` that is a valid AST
///
/// Returns: `(cons status result rest)`
/// where `status` can be one of the following:
///  * `ok`:         Success. `result` is the AST.
///  * `incomplete`: `input` is not a valid AST, but can be the beginning of a valid AST. `result` is undefined.
///  * `error`:      `input` is not a valid AST, not even the beginning of one. `result` contains the error details.
///  * `invalid`:    `input` is not a valid string. `result` is undefined.
/// `result` is the read AST and
/// `rest` is the unread rest of `input`.
pub fn read(mem: &mut Memory, input: ExternalReference) -> ExternalReference {
    let ok_sym         = mem.symbol_for("ok");
    let incomplete_sym = mem.symbol_for("incomplete");
    let invalid_sym    = mem.symbol_for("invalid");
    let error_sym      = mem.symbol_for("error");
    let invalid        = mem.allocate_cons(invalid_sym, ExternalReference::nil());
    let incomplete     = mem.allocate_cons(incomplete_sym, ExternalReference::nil());

    let mut state      = State::WhiteSpace;
    let mut list_stack = Vec::<Vec<ExternalReference>>::new();
    let mut atom_stack = Vec::<char>::new();
    let mut cursor     = input;

    while !cursor.is_nil() {
        let cons =
        if let PrimitiveValue::Cons(cons) = cursor.get() {
            cons
        }
        else {
            return invalid;
        };
        let ch =
        if let PrimitiveValue::Character(ch) = cons.get_car().get() {
            *ch
        }
        else {
            return invalid;
        };


        match state {
            State::WhiteSpace => {
                match ch {
                    ';' => state = State::Comment,
                    '"' => state = State::Comment,
                    '(' => list_stack.push(vec![]),
                    ')' => {
                        if read_list(mem, &mut list_stack).is_none() {
                            return incomplete;
                        }
                    },
                    c   => {
                        state = State::Atom;
                        atom_stack.push(c);
                    },
                }
            },
            State::Comment => {
                match ch {
                    '\n' => state = State::WhiteSpace,
                    _    => { /* do nothing */ },
                }
            },
            State::Atom => {
                if ch.is_whitespace() || ch == ';' || ch == '(' || ch == ')' {
                    if let Some(atom) = read_atom(mem, &mut list_stack, &mut atom_stack) {
                        let r = mem.allocate_cons(atom, cursor);
                        return mem.allocate_cons(ok_sym, r);
                    }

                    if ch.is_whitespace() {
                        state = State::WhiteSpace;
                    }
                    else if ch == ';' {
                        state = State::Comment;
                    }
                    else if ch == '(' {
                        list_stack.push(vec![]);
                        state = State::WhiteSpace;
                    }
                    else if ch == ')' {
                        if read_list(mem, &mut list_stack).is_none() {
                            return incomplete;
                        }
                        state = State::WhiteSpace;
                    }
                }
                else {
                    atom_stack.push(ch);
                }
            },
            State::String => {
                match ch {
                    '"' => {
                        if read_list(mem, &mut list_stack).is_none() {
                            return incomplete;
                        }
                        state = State::WhiteSpace;
                    },
                    '\\' => {
                        state = State::StringEscape;
                    }
                    c => list_stack.last_mut().unwrap().push(mem.allocate_character(c)),
                }
            },
            State::StringEscape => {
                let escaped;
                match ch {
                    'n'  => escaped = '\n',
                    't'  => escaped = '\r',
                    'r'  => escaped = '\r',
                    '"'  => escaped = '"',
                    '\\' => escaped = '\\',
                    _ => {
                        let error_msg = ExternalReference::nil(); // TODO
                        let error = mem.allocate_cons(error_sym, error_msg);
                        return error;
                    },
                }
                list_stack.last_mut().unwrap().push(mem.allocate_character(escaped));
                state = State::String;
            },
        }


        cursor = cons.get_cdr();
    }

    
    todo!()
}


enum State {
    WhiteSpace,
    Comment,
    String,
    StringEscape,
    Atom,
}


fn read_list(mem: &mut Memory, list_stack: &mut Vec<Vec<ExternalReference>>) -> Option<()> {
    let current_list = list_stack.pop()?;
    let mut list = ExternalReference::nil();

    for x in current_list.iter().rev() {
        list = mem.allocate_cons(x.clone(), list);
    }

    list_stack.last_mut().unwrap().push(list);

    Some(())
}


fn read_atom(mem: &mut Memory, list_stack: &mut Vec<Vec<ExternalReference>>, atom_stack: &mut Vec<char>) -> Option<ExternalReference> {
    let atom_str = atom_stack.iter().collect::<String>();
    atom_stack.clear();

    let atom = 
    if let Some(x) = read_number(mem, &atom_str) {
        x
    }
    else if let Some(x) = read_character(mem, &atom_str) {
        x
    }
    else {
        read_symbol(mem, &atom_str)
    };

    if let Some(current_list) = list_stack.last_mut() {
        current_list.push(atom);
        None
    }
    else {
        Some(atom)
    }
}


fn read_number(mem: &mut Memory, string: &str) -> Option<ExternalReference> {
    if let Ok(x) = string.parse() {
        Some(mem.allocate_number(x))
    }
    else {
        None
    }
}

fn read_character(mem: &mut Memory, string: &str) -> Option<ExternalReference> {
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

fn read_symbol(mem: &mut Memory, string: &str) -> ExternalReference {
    mem.symbol_for(&string)
}


#[cfg(test)]
mod tests;

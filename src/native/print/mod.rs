use crate::util::*;
use crate::memory::*;
use crate::error_utils::*;
use crate::config;
use super::NativeFunctionMetaData;


fn print_atom(mem: &mut Memory, atom: GcRef) -> GcRef {
    if atom.is_nil() {
        return string_to_list(mem, "()");
    }
    
    match atom.get().unwrap() {
        PrimitiveValue::Nil          => string_to_list(mem, "()"),
        PrimitiveValue::Number(x)    => string_to_list(mem, &format!("{x}")),
        PrimitiveValue::Character(x) => {
            let y =
            match x {
                '\t'  => "\\t".to_string(),
                '\n'  => "\\n".to_string(),
                '\r'  => "\\r".to_string(),
                '\\' => "\\\\".to_string(),
                _    => format!("{x}"),
            };
            string_to_list(mem, &format!("%{y}"))
        },
        PrimitiveValue::Symbol(x)    => string_to_list(mem, &format!("{}", x.get_name())),
        PrimitiveValue::Trap(_)      => string_to_list(mem, &format!("#<trap>")),
        PrimitiveValue::Function(f)  => {
            match f.get_kind() {
                FunctionKind::Lambda        => string_to_list(mem, &format!("#<lambda>")),
                FunctionKind::SpecialLambda => string_to_list(mem, &format!("#<special-lambda>")),
                FunctionKind::Macro         => string_to_list(mem, &format!("#<macro>")),
                FunctionKind::Syntax        => string_to_list(mem, &format!("#<syntax-macro>")),
            }
        },
        PrimitiveValue::Cons(x)      => {
            let car = list_to_string(print_atom(mem, x.get_car())).unwrap();
            let cdr = list_to_string(print_atom(mem, x.get_cdr())).unwrap();
            string_to_list(mem, &format!("(cons {car} {cdr})"))
        },
    }
}

fn print_string(mem: &mut Memory, string: String) -> GcRef {
    let mut result = string_to_list(mem, "\"");
    for c in string.chars().rev() {
        let character = mem.allocate_character(c);
        result = mem.allocate_cons(character, result);
    }

    let quote = mem.allocate_character('"');
    mem.allocate_cons(quote, result)
}

fn print_list(mem: &mut Memory, list: Vec<GcRef>) -> GcRef {
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


fn print_internal(mem: &mut Memory, expression: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", PRINT.name, &vec![]));
    }

    if expression.is_nil() {
        Ok(string_to_list(mem, "()"))
    }
    else if let Some(string) = list_to_string(expression.clone()) {
        Ok(print_string(mem, string))
    }
    else if let Some(mut elems) = list_to_vec(expression.clone()) {
        for elem in elems.iter_mut() {
            *elem = print_internal(mem, elem.clone(), recursion_depth + 1)?;
        }
        Ok(print_list(mem, elems))
    }
    else {
        Ok(print_atom(mem, expression))
    }
}


pub const PRINT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      print,
    name:          "print",
    kind:          FunctionKind::Lambda,
    parameters:    &["input"],
    documentation: "Convert `input` to its string representation.",
};

pub fn print(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, PRINT.name, args, (let x: TypeLabel::Any));

    print_internal(mem, x, recursion_depth + 1)
}


pub fn print_to_rust_string(expression: GcRef, recursion_depth: usize) -> Result<String, String> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(format!("stackoverflow"));
    }

    if expression.is_nil() {
        Ok(format!("()"))
    }
    else if let Some(string) = list_to_string(expression.clone()) {
        Ok(format!("\"{string}\""))
    }
    else if let Some(elems) = list_to_vec(expression.clone()) {
        let mut strings = vec![format!("(")];
        for elem in elems.iter() {
            strings.push(print_to_rust_string(elem.clone(), recursion_depth + 1)?);
        }
        strings.push(format!(")"));
        Ok(strings.join(" "))
    }
    else {
        if expression.is_nil() {
            return Ok(format!("()"));
        }

        match expression.get().unwrap() {
            PrimitiveValue::Nil          => Ok(format!("()")),
            PrimitiveValue::Number(x)    => Ok(format!("{x}")),
            PrimitiveValue::Character(x) => {
                let y =
                match x {
                    '\t'  => "\\t".to_string(),
                    '\n'  => "\\n".to_string(),
                    '\r'  => "\\r".to_string(),
                    '\\' => "\\\\".to_string(),
                    _    => format!("{x}"),
                };
                Ok(format!("%{y}"))
            },
            PrimitiveValue::Symbol(x)    => Ok(format!("{}", x.get_name())),
            PrimitiveValue::Trap(_)      => Ok(format!("#<trap>")),
            PrimitiveValue::Function(f)  => {
                match f.get_kind() {
                    FunctionKind::Lambda        => Ok(format!("#<lambda>")),
                    FunctionKind::SpecialLambda => Ok(format!("#<special-lambda>")),
                    FunctionKind::Macro         => Ok(format!("#<macro>")),
                    FunctionKind::Syntax        => Ok(format!("#<syntax-macro>")),
                }
            },
            PrimitiveValue::Cons(x)      => {
                let car = print_to_rust_string(x.get_car(), recursion_depth + 1)?;
                let cdr = print_to_rust_string(x.get_cdr(), recursion_depth + 1)?;
                Ok(format!("(cons {car} {cdr})"))
            },
        }
    }
}



#[cfg(test)]
mod tests;

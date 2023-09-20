#![allow(dead_code)]

use crate::memory::*;
use crate::util::*;
use crate::native::list::make_plist;
use crate::error_utils::*;
use crate::parser::*;
use super::NativeFunctionMetaData;
use std::path::PathBuf;



fn whitespace_character(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        let ch = x.get().unwrap().as_character();
        ch.is_whitespace() || *ch == ','
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn whitespace(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    (at_least_once(&whitespace_character))(mem, input)
}


fn comment_start(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        *x.get().unwrap().as_character() == ';'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn inside_comment(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        *x.get().unwrap().as_character() != '\n'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn comment(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let start = comment_start(mem, input)?;

    (zero_or_more_times(&inside_comment))(mem, start.rest_of_input)
}


fn number_sign(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        let ch = x.get().unwrap().as_character();
        *ch == '-' || *ch == '+'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn digit(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        x.get().unwrap().as_character().is_digit(10)
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn number(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let default_sign = mem.allocate_character('+');
    let sign         = (parse_or_default(&number_sign, &default_sign))(mem, input.clone())?;
    let sign_n       = if *sign.value.get().unwrap().as_character() == '+' { 1 } else { -1 };

    let lisp_string = (at_least_once(&digit))(mem, sign.rest_of_input)?;
    let rust_string = list_to_vec(lisp_string.value).unwrap().iter().map(|x| *x.get().unwrap().as_character()).collect::<String>();
    let rust_number = rust_string.parse::<i64>().unwrap();
    let lisp_number = mem.allocate_number(rust_number * sign_n);
    let meta        = Metadata{ read_name: rust_string, location: input.location.clone(), documentation: "".to_string(), parameters: vec![] };

    Ok(ParserOk{ value: mem.allocate_metadata(lisp_number, meta), location: input.location, rest_of_input: lisp_string.rest_of_input })
}


fn character_start(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        *x.get().unwrap().as_character() == '%'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn escape_character(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        let ch = x.get().unwrap().as_character();
        *ch == 'n' || *ch == 'r' || *ch == 't' || *ch == '\\'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn character(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let start = character_start(mem, input.clone())?;

    let x1 = any_character(mem, start.rest_of_input)?;
    let c1 = x1.value.get().unwrap().as_character(); 
    
    let ch;
    let rest;
    if *c1 == '\\' {
        let x2 = escape_character(mem, x1.rest_of_input)?;
        let c2 = x2.value.get().unwrap().as_character(); 
        ch =
        match c2 {
            'n'  => '\n',
            'r'  => '\r',
            't'  => '\t',
            '\\' => '\\',
            _    => unreachable!()
        };
        rest = x2.rest_of_input;
    }
    else {
        ch   = *c1;
        rest = x1.rest_of_input;
    };


    let lisp_ch = mem.allocate_character(ch);
    let meta    = Metadata{ read_name: format!("%{ch}"), location: input.location.clone(), documentation: "".to_string(), parameters: vec![] };

    Ok(ParserOk{ value: mem.allocate_metadata(lisp_ch, meta), location: input.location, rest_of_input: rest })
}


fn symbol_char(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        let ch = x.get().unwrap().as_character();
        *ch != '(' && *ch != ')' && *ch != '"' && !ch.is_whitespace() && *ch != ','
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn symbol(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let lisp_string = (at_least_once(&symbol_char))(mem, input.clone())?;
    let rust_string = list_to_vec(lisp_string.value).unwrap().iter().map(|x| *x.get().unwrap().as_character()).collect::<String>();
    let sym         = mem.symbol_for(&rust_string);
    let meta        = Metadata{ read_name: rust_string, location: input.location.clone(), documentation: "".to_string(), parameters: vec![] };
    Ok(ParserOk{ value: mem.allocate_metadata(sym, meta), location: input.location, rest_of_input: lisp_string.rest_of_input })
}


fn atom(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    (one_of_these_three(&number, &character, &symbol))(mem, input.clone())
}


fn open_paren(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        *x.get().unwrap().as_character() == '('
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn close_paren(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        *x.get().unwrap().as_character() == ')'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn list(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let op  = open_paren(mem, input.clone())?;
    let lst = (zero_or_more_times(&expression))(mem, op.rest_of_input)?;
    let jnk = (zero_or_more_times(&junk))(mem, lst.rest_of_input)?;
    let cp  = close_paren(mem, jnk.rest_of_input)?;

    Ok(ParserOk{ value: lst.value, location: input.location, rest_of_input: cp.rest_of_input} )
}


fn quote(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        *x.get().unwrap().as_character() == '"'
    }

    (satisfy(&any_character, &p))(mem, input)
}


fn string_normal(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        let ch = x.get().unwrap().as_character();
        *ch != '"' && *ch != '\\'
    }

    (satisfy(&any_character, &p))(mem, input)
}

fn string_escape(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    fn p(x: GcRef) -> bool {
        let ch = x.get().unwrap().as_character();
        *ch == '\\'
    }

    let backslash = (satisfy(&any_character, &p))(mem, input.clone())?;
    let ech = any_character(mem, backslash.rest_of_input)?;

    let ch =
    match ech.value.get().unwrap().as_character() {
        '"'  => '"',
        'n'  => '\n',
        'r'  => '\t',
        '\\' => '\\',
        c    => return Err(ParserError::Fatal{ message: format!("'{c}' is not a valid escape character in a string literal"), location: ech.location, rest_of_input: ech.rest_of_input })
    };

    Ok(ParserOk{ value: mem.allocate_character(ch), location: input.location, rest_of_input: ech.rest_of_input} )
}

fn string_char(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    (one_of_these_two(&string_escape, &string_normal))(mem, input)
}


fn string(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let oq  = quote(mem, input.clone())?;
    let chs = (zero_or_more_times(&string_char))(mem, oq.rest_of_input)?;
    let cq  = quote(mem, chs.rest_of_input)?;

    let meta = Metadata{ read_name: "".to_string(), location: input.location.clone(), documentation: "".to_string(), parameters: vec![] };

    let list_symbol = mem.symbol_for("list");
    let the_string  = mem.allocate_cons(list_symbol, chs.value);
    Ok(ParserOk{ value: mem.allocate_metadata(the_string, meta), location: input.location, rest_of_input: cq.rest_of_input} )
}


fn junk(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    (one_of_these_two(&whitespace, &comment))(mem, input)
}


fn nothing(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    (one_of_these_two(&end_of_input, &junk))(mem, input)
}


fn expression(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    let jnk = (zero_or_more_times(&junk))(mem, input)?;
    (one_of_these_three(&string, &atom, &list))(mem, jnk.rest_of_input)
}


fn format_error(mem: &mut Memory, location: Location, msg: String, rest: GcRef, rest_line: usize, rest_column: usize) -> GcRef {
    let error_sym = mem.symbol_for("error");
    let error_msg = string_to_list(mem, &msg);
    let file;
    let line;
    let column;
    match location {
        Location::Native                             => unreachable!(),
        Location::Prelude{ line: ln, column: cn }    => {
            file = mem.symbol_for("prelude");
            line = mem.allocate_number(ln as i64);
            column = mem.allocate_number(cn as i64);
        },
        Location::Stdin{ line: ln, column: cn }      => {
            file = mem.symbol_for("stdin");
            line = mem.allocate_number(ln as i64);
            column = mem.allocate_number(cn as i64);
        },
        Location::File{ path, line: ln, column: cn } => {
            file = string_to_proper_list(mem, &path.into_os_string().into_string().unwrap());
            line = mem.allocate_number(ln as i64);
            column = mem.allocate_number(cn as i64);
        },
    }
    let error_loc = vec_to_list(mem, &vec![file, line, column]);
    let error     = make_plist(mem, &vec![("location", error_loc), ("message", error_msg)]);
    let ln        = mem.allocate_number(rest_line as i64);
    let cn        = mem.allocate_number(rest_column as i64);
    make_plist(mem, &vec![("status", error_sym), ("error", error), ("rest", rest), ("line", ln), ("column", cn)])
}


pub const READ: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      read,
    name:          "read",
    kind:          FunctionKind::Lambda,
    parameters:    &["input", "&", "source", "start-line", "start-column"],
    documentation: 
r"Converts a Lisp-style string to an AST.

Only reads the shortest prefix of the input string that is a valid AST.

Returns a property list which always contains at least a `status` key.
The `status` key can have one of the following values:
 * `ok`:         Success. The key `result` is the AST.
 * `nothing`:    The input was empty or only contained whitespace.
 * `incomplete`: The input is not a valid AST, but can be the beginning of a valid AST.
 * `error`:      The input is not a valid AST, not even the beginning of one. The `error` key contains the error details.
 * `invalid`:    The input is not a valid string.

Whenever there is a `rest` key, the `line` and `column` keys are also present,
whose values are respectively the first line and column of the rest of the input.

`source`, `start-line` and `start-column` describe where we are reading from.
Possible values of `source`:
 * prelude
 * stdin
 * a string representing a file-path.",
};

pub fn read(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if args.len() != 1 && args.len() != 4 {
        let vec           = vec![mem.symbol_for("or"), mem.allocate_number(1), mem.allocate_number(4)];
        let error_details = vec![("expected", vec_to_list(mem, &vec)), ("actual", fit_to_number(mem, args.len()))];
        let error         = make_error(mem, "wrong-number-of-arguments", READ.name, &error_details);
        return Err(error);
    }

    let start_line;
    let start_column;
    let location;
    if args.len() == 4 {
        if let Some(PrimitiveValue::Number(x)) = args[2].get() {
            start_line = *x as usize;
        }
        else {
            let actual_type = crate::native::reflection::type_of(mem, &[args[2].clone()], GcRef::nil(), recursion_depth + 1)?;
            let error_details = vec![("expected", mem.symbol_for("number")), ("actual", actual_type)];
            let error = make_error(mem, "wrong-argument-type", READ.name, &error_details);
            return Err(error);
        }

        if let Some(PrimitiveValue::Number(x)) = args[3].get() {
            start_column = *x as usize;
        }
        else {
            let actual_type = crate::native::reflection::type_of(mem, &[args[3].clone()], GcRef::nil(), recursion_depth + 1)?;
            let error_details = vec![("expected", mem.symbol_for("number")), ("actual", actual_type)];
            let error = make_error(mem, "wrong-argument-type", READ.name, &error_details);
            return Err(error);
        }

        if let Some(path) = list_to_string(args[1].clone()) {
            location = Location::File { path: PathBuf::from(path), line: start_line, column: start_column };
        }
        else if symbol_eq!(args[1], mem.symbol_for("prelude")) {
            location = Location::Prelude { line: start_line, column: start_column };
        }
        else if symbol_eq!(args[1], mem.symbol_for("stdin")) {
            location = Location::Stdin { line: start_line, column: start_column };
        }
        else {
            let error_details = vec![("the-unknown-source", args[1].clone())];
            let error = make_error(mem, "unknown-read-source", READ.name, &error_details);
            return Err(error);
        }
    }
    else {
        location = Location::Stdin { line: 1, column: 1 };
    }

    let input = args[0].clone();

    match close_paren(mem, StringWithLocation { string: input.clone(), location: location.clone() }) {
        Ok(output) => {
            let rest = output.rest_of_input.trim();
            return Ok(format_error(mem, rest.location.clone(), "too many closing parentheses".to_string(), rest.string, rest.location.get_line().unwrap(), rest.location.get_column().unwrap()))
        }
        Err(_) => {
            // continue
        }
    }


    let result =
    match expression(mem, StringWithLocation { string: input.clone(), location: location.clone() }) {
        Ok(output) => {
            let rest = output.rest_of_input.trim();
            let mut vec = vec![("status", mem.symbol_for("ok")), ("result", output.value), ("rest", rest.string)];
            if let Some(line) = rest.location.get_line() {
                vec.push(("line", fit_to_number(mem, line)));
            }
            if let Some(column) = rest.location.get_column() {
                vec.push(("column", fit_to_number(mem, column)));
            }
            make_plist(mem, &vec)
        },
        Err(err)   => {
            match err {
                ParserError::NoMatch => {
                    let vec = vec![("status", mem.symbol_for("incomplete"))];
                    make_plist(mem, &vec)
                },
                ParserError::Incomplete => {
                    match nothing(mem, StringWithLocation { string: input, location}) {
                        Ok(_) => {
                            let vec = vec![("status", mem.symbol_for("nothing"))];
                            make_plist(mem, &vec)
                        },
                        Err(err) => {
                            match err {
                                ParserError::NoMatch | ParserError::Incomplete => {
                                    let vec = vec![("status", mem.symbol_for("incomplete"))];
                                    make_plist(mem, &vec)
                                }
                                ParserError::Fatal{ message, location, rest_of_input } => {
                                    format_error(mem, location, message, rest_of_input.string, rest_of_input.location.get_line().unwrap(), rest_of_input.location.get_column().unwrap())
                                }
                            }
                        },
                    }
                }
                ParserError::Fatal{ message, location, rest_of_input } => {
                    format_error(mem, location, message, rest_of_input.string, rest_of_input.location.get_line().unwrap(), rest_of_input.location.get_column().unwrap())
                }
            }
        },
    };

    Ok(result)
}


#[cfg(test)]
mod tests;

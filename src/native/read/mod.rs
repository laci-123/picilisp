#![allow(dead_code)]

use crate::memory::*;
use crate::metadata::*;
use crate::util::*;
use crate::native::list::make_plist;
use crate::error_utils::*;
use crate::config;
use super::NativeFunctionMetaData;
use std::path::PathBuf;
use std::iter::Peekable;



enum TokenValue {
    OpenParen,
    CloseParen,
    Character(char),
    Number(i64),
    Symbol(String),
    Quote,
    String(String),
    Incomplete,
}

struct Token {
    value: TokenValue,
    location: Location,
}


struct TokenAndRest {
    token: Token,
    rest: GcRef,
}

impl TokenAndRest {
    fn new(value: TokenValue, location: Location, rest: GcRef) -> Self {
        Self {
            token: Token { value, location },
            rest
        }
    }
}


#[derive(PartialEq, Eq)]
enum TokenIteratorStatus {
    WhiteSpace,
    Comment,
    Character,
    Number,
    Symbol,
}


struct TokenIterator {
    input: Peekable<StringIterator>,
    location: Location,
}

impl TokenIterator {
    fn new(input: GcRef, location: Location) -> Self {
        Self{ input: StringIterator::new(input).peekable(), location }
    }
}

impl Iterator for TokenIterator {
    type Item = Result<TokenAndRest, String>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenIteratorStatus::*;

        let mut status = WhiteSpace;
        let mut buffer = vec![];

        while let Some(maybe_char_and_rest) = self.input.next() {
            let (ch, rest) = if let Some(x) = maybe_char_and_rest {x} else {return Some(Err("invalid-string".to_string()));};

            if ch == '\n' {
                self.location = self.location.clone().step_line();
            }
            else {
                self.location = self.location.clone().step_column();
            }

            if status == Comment {
                if ch == '\n' {
                    status = WhiteSpace;
                }
                continue;
            }
            
            match ch {
                c if c.is_whitespace() || c == ',' => {
                    status = WhiteSpace;
                },
                ';' => {
                    status = Comment;
                },
                '\'' => {
                    return Some(Ok(TokenAndRest::new(TokenValue::Quote, self.location.clone(), rest)));
                },
                '(' => {
                    return Some(Ok(TokenAndRest::new(TokenValue::OpenParen, self.location.clone(), rest)));
                },
                ')' => {
                    return Some(Ok(TokenAndRest::new(TokenValue::CloseParen, self.location.clone(), rest)));
                },
                '%' => {
                    if status == WhiteSpace {
                        status = Character;
                    }
                },
                c if c.is_ascii_digit() || c == '+' || c == '-' => {
                    if status == WhiteSpace {
                        status = Number;
                    }
                    buffer.push(ch);
                }
                _ => {
                    if status == WhiteSpace {
                        status = Symbol;
                    }
                    buffer.push(ch);
                },
            }

            if buffer.len() > 0 {
                let next_ch = if let Some(x) = self.input.peek() {x} else {return Some(Err("invalid-string".to_string()));};
                if is_atom_ending(next_ch) {
                    let token_and_rest =
                    match status {
                        Character => build_character(&buffer).map(|x| TokenAndRest::new(TokenValue::Character(x), self.location.clone(), rest)),
                        Number    => build_number(&buffer).map(   |x| TokenAndRest::new(TokenValue::Number(x),    self.location.clone(), rest)),
                        Symbol    => build_symbol(&buffer).map(   |x| TokenAndRest::new(TokenValue::Symbol(x),    self.location.clone(), rest)),
                        _         => unreachable!(),
                    };
                    return Some(token_and_rest);
                }
            }
        }
        
        if buffer.len() > 0 {
            Some(Ok(TokenAndRest::new(TokenValue::Incomplete, self.location.clone(), GcRef::nil())))
        }
        else {
            None
        }
    }
}


fn is_atom_ending(ch: &Option<(char, GcRef)>) -> bool {
    match ch {
        None => true,
        Some((c, _)) => {
            match c {
                ';' | '(' | ')' | '\'' | ',' => true,
                k if k.is_whitespace() => true,
                _ => false,
            } 
        }
    }
}


fn build_character(chars: &[char]) -> Result<char, String> {
    match chars.iter().collect::<String>().as_str() {
        ""   => Err(format!("invalid character: '%' (empty literal)")),
        "\\n" => Ok('\n'),
        "\\t" => Ok('\t'),
        "\\r" => Ok('\r'),
        "\\\\" => Ok('\\'),
        c if c.len() == 1 => Ok(c.chars().next().unwrap()),
        c    => Err(format!("invalid character: '%{c}'")),
    }
}


fn build_symbol(chars: &[char]) -> Result<String, String> {
    Ok(chars.iter().collect())
}


fn build_number(chars: &[char]) -> Result<i64, String> {
    chars.iter().collect::<String>().parse::<i64>().map_err(|err| err.to_string())
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
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", READ.name, &vec![]));
    }

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

    todo!()
}


#[cfg(test)]
mod tests;

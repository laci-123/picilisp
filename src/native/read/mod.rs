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
    String(String),
}

struct Token {
    value: TokenValue,
    location: Location,
    quoted: bool,
}


struct TokenAndRest {
    token: Token,
    rest: GcRef,
}

impl TokenAndRest {
    fn new(value: TokenValue, location: Location, rest: GcRef, quoted: bool) -> Self {
        Self {
            token: Token { value, location, quoted },
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
    StringStatus,
    StringEscape,
}


enum ReadError {
    InvalidString,
    Incomplete,
    Nothing,
    Error{ msg: String, location: Location, rest: GcRef },
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
    type Item = Result<TokenAndRest, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenIteratorStatus::*;

        let mut status = WhiteSpace;
        let mut buffer = vec![];
        let mut quoting = false;

        while let Some(maybe_char_and_rest) = self.input.next() {
            let (ch, rest) = if let Some(x) = maybe_char_and_rest {x} else {return Some(Err(ReadError::InvalidString));};

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
                    quoting = true;
                },
                '(' => {
                    return Some(Ok(TokenAndRest::new(TokenValue::OpenParen, self.location.clone(), rest, quoting)));
                },
                ')' => {
                    return Some(Ok(TokenAndRest::new(TokenValue::CloseParen, self.location.clone(), rest, quoting)));
                },
                '"' => {
                    match status {
                        StringEscape => {
                            buffer.push('"');
                        },
                        StringStatus => {
                            return Some(Ok(TokenAndRest::new(TokenValue::String(buffer.iter().collect()), self.location.clone(), rest, quoting)));
                        }
                        _ => {
                            status = StringStatus;
                        }
                    }
                },
                '\\' => {
                    if status == StringStatus {
                        status = StringEscape;
                    }
                    else {
                        return Some(Err(ReadError::Error{ msg: format!("unexpected character: '\\'"), location: self.location.clone(), rest }));
                    }
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
                    match status {
                        WhiteSpace => {
                            status = Symbol;
                            buffer.push(ch);
                        },
                        StringEscape => {
                            match ch {
                                'n'  => buffer.push('\n'),
                                'r'  => buffer.push('\r'),
                                't'  => buffer.push('\t'),
                                '\\' => buffer.push('\\'),
                                _    => buffer.push(ch),
                            }
                        }
                        _ => buffer.push(ch),
                    }
                },
            }

            if buffer.len() > 0 {
                let next_ch = if let Some(x) = self.input.peek() {x} else {return Some(Err(ReadError::InvalidString));};
                if is_atom_ending(next_ch) {
                    let token_and_rest =
                    match status {
                        Character => build_character(&buffer, self.location.clone(), rest.clone()).map(|x| TokenAndRest::new(TokenValue::Character(x), self.location.clone(), rest, quoting)),
                        Number    => build_number(&buffer, self.location.clone(), rest.clone()).map(   |x| TokenAndRest::new(TokenValue::Number(x),    self.location.clone(), rest, quoting)),
                        Symbol    => build_symbol(&buffer, self.location.clone(), rest.clone()).map(   |x| TokenAndRest::new(TokenValue::Symbol(x),    self.location.clone(), rest, quoting)),
                        _         => unreachable!(),
                    };
                    return Some(token_and_rest);
                }
            }
        }
        
        if buffer.len() > 0 || quoting {
            Some(Err(ReadError::Incomplete))
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
                ';' | '(' | ')' | '"' | '\'' | ',' => true,
                k if k.is_whitespace()             => true,
                _                                  => false,
            } 
        }
    }
}


fn build_character(chars: &[char], location: Location, rest: GcRef) -> Result<char, ReadError> {
    match chars.iter().collect::<String>().as_str() {
        ""   => Err(ReadError::Error{ msg: format!("invalid character: '%' (empty literal)"), location, rest }),
        "\\n" => Ok('\n'),
        "\\t" => Ok('\t'),
        "\\r" => Ok('\r'),
        "\\\\" => Ok('\\'),
        c if c.len() == 1 => Ok(c.chars().next().unwrap()),
        c    => Err(ReadError::Error{ msg: format!("invalid character: '%{c}'"), location, rest }),
    }
}


fn build_symbol(chars: &[char], _location: Location, _rest: GcRef) -> Result<String, ReadError> {
    Ok(chars.iter().collect())
}


fn build_number(chars: &[char], location: Location, rest: GcRef) -> Result<i64, ReadError> {
    chars.iter().collect::<String>().parse::<i64>().map_err(|err| ReadError::Error{ msg: err.to_string(), location, rest })
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


fn read_internal(mem: &mut Memory, input: GcRef, location: Location) -> Result<(GcRef, GcRef), ReadError> {
    let mut stack = vec![];
    
    for maybe_token_and_rest in TokenIterator::new(input, location) {
        let TokenAndRest{token, rest} = maybe_token_and_rest?;

        let result;
        match token.value {
            TokenValue::OpenParen => {
                stack.push(vec![]);
                continue;
            },
            TokenValue::CloseParen => {
                if let Some(vec) = stack.pop() {
                    let list = vec_to_list(mem, &vec);
                    if let Some(lower_vec) = stack.last_mut() {
                        lower_vec.push(list);
                        continue;
                    }
                    else {
                        result = list;
                    }
                }
                else {
                    return Err(ReadError::Error{ msg: format!("too many closing parentheses"), location: token.location, rest });
                }
            },
            TokenValue::Character(c) => {
                let x = mem.allocate_character(c).with_metadata(Metadata{ read_name: format!("{c}"), location: token.location, documentation: String::new() });
                if let Some(vec) = stack.last_mut() {
                    vec.push(x);
                    continue;
                }
                else {
                    result = x;
                }
            },
            TokenValue::Number(n) => {
                let x = mem.allocate_number(n).with_metadata(Metadata{ read_name: format!("{n}"), location: token.location, documentation: String::new() });
                if let Some(vec) = stack.last_mut() {
                    vec.push(x);
                    continue;
                }
                else {
                    result = x;
                }
            },
            TokenValue::Symbol(s) => {
                let x = mem.symbol_for(s.as_str()).with_metadata(Metadata{ read_name: format!("{s}"), location: token.location, documentation: String::new() });
                if let Some(vec) = stack.last_mut() {
                    vec.push(x);
                    continue;
                }
                else {
                    result = x;
                }
            },
            TokenValue::String(s) => {
                let x = string_to_proper_list(mem, s.as_str()).with_metadata(Metadata{ read_name: String::new(), location: token.location, documentation: String::new() });
                if let Some(vec) = stack.last_mut() {
                    vec.push(x);
                    continue;
                }
                else {
                    result = x;
                }
            },
        }

        if token.quoted {
            let vec = vec![mem.symbol_for("quote"), result];
            return Ok((vec_to_list(mem, &vec), rest));
        }
        else {
            return Ok((result, rest));
        }
    }

    if stack.len() > 0 {
        Err(ReadError::Incomplete)
    }
    else {
        Err(ReadError::Nothing)
    }
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

    match read_internal(mem, input, location) {
        Ok((result, rest)) => {
            let kv = vec![("status", mem.symbol_for("ok")), ("result", result), ("rest", rest)];
            Ok(make_plist(mem, &kv))
        },
        Err(ReadError::Nothing) => {
            let kv = vec![("status", mem.symbol_for("nothing"))];
            Ok(make_plist(mem, &kv))
        },
        Err(ReadError::Incomplete) => {
            let kv = vec![("status", mem.symbol_for("incomplete"))];
            Ok(make_plist(mem, &kv))
        },
        Err(ReadError::InvalidString) => {
            let kv = vec![("status", mem.symbol_for("invalid"))];
            Ok(make_plist(mem, &kv))
        },
        Err(ReadError::Error{ msg, location, rest }) => {
            Err(format_error(mem, location, msg, rest, 0, 0))
        },
    }
}


#[cfg(test)]
mod tests;

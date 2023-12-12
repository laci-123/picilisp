use crate::memory::*;
use crate::metadata::*;
use crate::util::*;
use crate::native::list::make_plist;
use crate::error_utils::*;
use crate::config;
use super::NativeFunctionMetaData;
use unicode_segmentation::UnicodeSegmentation;
use std::path::PathBuf;
use std::iter::Peekable;



enum TokenValue {
    OpenParen,
    CloseParen,
    Character(char),
    Number(i64),
    Symbol(String),
    String(String),
    Quote,
}


struct Token {
    value: TokenValue,
    location: Location,
}


#[derive(Clone)]
struct StringWithPosition {
    string: GcRef,
    line: usize,
    column: usize,
}

impl StringWithPosition {
    fn new(string: GcRef, line: usize, column: usize) -> Self {
        Self{ string, line, column }
    }
}


struct TokenAndRest {
    token: Token,
    rest: StringWithPosition,
}

impl TokenAndRest {
    fn new(value: TokenValue, location: Location, rest: StringWithPosition) -> Self {
        Self {
            token: Token { value, location },
            rest,
        }
    }
}


#[derive(PartialEq, Eq, Debug)]
enum TokenIteratorStatus {
    WhiteSpace,
    Comment,
    Character,
    Number,
    Symbol,
    SymbolOrNumber,
    StringNormal,
    StringEscape,
}


enum ReadError {
    InvalidString,
    Incomplete,
    Nothing,
    Error{ msg: String, location: Location, rest: StringWithPosition },
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
        let mut beginning_location = self.location.clone();

        while let Some(maybe_char_and_rest) = self.input.next() {
            let (ch, r) = if let Some(x) = maybe_char_and_rest {x} else {return Some(Err(ReadError::InvalidString));};

            if ch == '\n' {
                self.location.step_line();
            }
            else {
                self.location.step_column();
            }

            let rest = StringWithPosition::new(r, self.location.get_line().unwrap(), self.location.get_column().unwrap() + 1);


            if status == Comment {
                if ch == '\n' {
                    status = WhiteSpace;
                }
                continue;
            }
            if status == StringNormal {
                if ch != '"' && ch != '\\' {
                    buffer.push(ch);
                    continue;
                }
            }
            if status == StringEscape {
                match ch {
                    '"'  => buffer.push('"'),
                    'n'  => buffer.push('\n'),
                    'r'  => buffer.push('\r'),
                    't'  => buffer.push('\t'),
                    '\\' => buffer.push('\\'),
                    _    => return Some(Err(ReadError::Error{ msg: format!("'{ch}' is not a valid escape character in a string literal"), location: self.location.clone(), rest })),
                }
                status = StringNormal;
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
                '"' => {
                    match status {
                        StringNormal => {
                            return Some(Ok(TokenAndRest::new(TokenValue::String(buffer.iter().collect()), beginning_location, rest)));
                        }
                        _ => {
                            status = StringNormal;
                            beginning_location = self.location.clone();
                        }
                    }
                },
                '\\' => {
                    match status {
                        StringNormal => {
                            status = StringEscape;
                        },
                        Character => {
                            buffer.push(ch);
                        },
                        _ => {
                            return Some(Err(ReadError::Error{ msg: format!("unexpected character: '\\'"), location: self.location.clone(), rest }));
                        }
                    }
                },
                '%' => {
                    if status == WhiteSpace {
                        status = Character;
                        beginning_location = self.location.clone();
                    }
                    else {
                        buffer.push(ch);
                    }
                },
                '+' | '-' => {
                    if status == WhiteSpace {
                        status = SymbolOrNumber;
                        beginning_location = self.location.clone();
                    }
                    buffer.push(ch);
                },
                c if c.is_ascii_digit() => {
                    match status {
                        SymbolOrNumber => {
                            status = Number;
                        },
                        WhiteSpace => {
                            status = Number;
                            beginning_location = self.location.clone();
                        },
                        _ => {},
                    }
                    buffer.push(ch);
                },
                _ => {
                    match status {
                        WhiteSpace => {
                            status = Symbol;
                            beginning_location = self.location.clone();
                        },
                        SymbolOrNumber => {
                            status = Symbol;
                        },
                        Number => {
                            return Some(Err(ReadError::Error{ msg: format!("unexpected character in number literal: '{ch}'"), location: self.location.clone(), rest }));
                        },
                        _ => {},
                    }
                    buffer.push(ch);
                },
            }

            if buffer.len() > 0 {
                let atom_ending = if let Some(x) = is_atom_ending(&self.input.peek()) {x} else {return Some(Err(ReadError::InvalidString));};
                if atom_ending {
                    match status {
                        Character                   => return Some(build_character(&buffer, self.location.clone(), rest.clone()).map(|x| TokenAndRest::new(TokenValue::Character(x), beginning_location, rest))),
                        Number                      => return Some(build_number(   &buffer, self.location.clone(), rest.clone()).map(|x| TokenAndRest::new(TokenValue::Number(x),    beginning_location, rest))),
                        Symbol | SymbolOrNumber     => return Some(build_symbol(   &buffer, self.location.clone(), rest.clone()).map(|x| TokenAndRest::new(TokenValue::Symbol(x),    beginning_location, rest))),
                        StringNormal | StringEscape => { /* don't do anything */ },
                        _                           => unreachable!(),
                    }
                }
            }
        }
        
        if buffer.len() > 0 {
            Some(Err(ReadError::Incomplete))
        }
        else {
            None
        }
    }
}


fn is_atom_ending(ch: &Option<&Option<(char, GcRef)>>) -> Option<bool> { // None: invalid string
    match ch {
        None => Some(true),
        Some(Some((c, _))) => {
            match c {
                ';' | '(' | ')' | '"' | '\'' | ',' => Some(true),
                k if k.is_whitespace()             => Some(true),
                _                                  => Some(false),
            } 
        },
        Some(None) => None,
    }
}


fn build_character(chars: &[char], location: Location, rest: StringWithPosition) -> Result<char, ReadError> {
    match chars.iter().collect::<String>().as_str() {
        ""     => Err(ReadError::Error{ msg: format!("invalid character: '%' (empty literal)"), location, rest }),
        "\\n"  => Ok('\n'),
        "\\t"  => Ok('\t'),
        "\\s"  => Ok(' '),
        "\\r"  => Ok('\r'),
        "\\\\" => Ok('\\'),
        c if c.graphemes(true).count() == 1 => Ok(c.chars().next().unwrap()),
        c      => Err(ReadError::Error{ msg: format!("invalid character: '%{c}'"), location, rest }),
    }
}


fn build_symbol(chars: &[char], _location: Location, _rest: StringWithPosition) -> Result<String, ReadError> {
    Ok(chars.iter().collect())
}


fn build_number(chars: &[char], location: Location, rest: StringWithPosition) -> Result<i64, ReadError> {
    chars.iter().collect::<String>().parse::<i64>().map_err(|err| ReadError::Error{ msg: format!("invalid number: '{err}'"), location, rest })
}



fn format_error(mem: &mut Memory, location: Location, msg: String, rest: StringWithPosition) -> GcRef {
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
    let ln        = mem.allocate_number(rest.line as i64);
    let cn        = mem.allocate_number(rest.column as i64);
    make_plist(mem, &vec![("status", error_sym), ("error", error), ("rest", rest.string), ("line", ln), ("column", cn)])
}


fn read_internal(mem: &mut Memory, input: GcRef, location: Location) -> Result<(GcRef, StringWithPosition), ReadError> {
    let mut stack = vec![];
    let mut quoted = false;

    for maybe_token_and_rest in TokenIterator::new(input, location) {
        let TokenAndRest{token, rest} = maybe_token_and_rest?;

        let result;
        match token.value {
            TokenValue::Quote => {
                quoted = true;
                continue;
            },
            TokenValue::OpenParen => {
                stack.push((vec![], quoted));
                quoted = false;
                continue;
            },
            TokenValue::CloseParen => {
                if let Some((vec, q)) = stack.pop() {
                    let list = vec_to_list(mem, &vec);
                    let qlist =
                    if q {
                        let vec = vec![mem.symbol_for("quote"), list];
                        quoted = false;
                        vec_to_list(mem, &vec)
                    }
                    else {
                        list
                    };
                    if let Some((lower_vec, _)) = stack.last_mut() {
                        lower_vec.push(qlist);
                        continue;
                    }
                    else {
                        result = qlist;
                    }
                }
                else {
                    return Err(ReadError::Error{ msg: format!("too many closing parentheses"), location: token.location, rest });
                }
            },
            TokenValue::Character(c) => {
                let x = mem.allocate_character(c).with_metadata(Metadata{ read_name: format!("{c}"), location: token.location, documentation: String::new() });
                let y =
                if quoted {
                    let vec = vec![mem.symbol_for("quote"), x];
                    quoted = false;
                    vec_to_list(mem, &vec)
                }
                else {
                    x
                };
                if let Some((vec, _)) = stack.last_mut() {
                    vec.push(y);
                    continue;
                }
                else {
                    result = y;
                }
            },
            TokenValue::Number(n) => {
                let x = mem.allocate_number(n).with_metadata(Metadata{ read_name: format!("{n}"), location: token.location, documentation: String::new() });
                let y =
                if quoted {
                    let vec = vec![mem.symbol_for("quote"), x];
                    quoted = false;
                    vec_to_list(mem, &vec)
                }
                else {
                    x
                };
                if let Some((vec, _)) = stack.last_mut() {
                    vec.push(y);
                    continue;
                }
                else {
                    result = y;
                }
            },
            TokenValue::Symbol(s) => {
                let x = mem.symbol_for(s.as_str()).with_metadata(Metadata{ read_name: format!("{s}"), location: token.location, documentation: String::new() });
                let y =
                if quoted {
                    let vec = vec![mem.symbol_for("quote"), x];
                    quoted = false;
                    vec_to_list(mem, &vec)
                }
                else {
                    x
                };
                if let Some((vec, _)) = stack.last_mut() {
                    vec.push(y);
                    continue;
                }
                else {
                    result = y;
                }
            },
            TokenValue::String(s) => {
                let x = string_to_proper_list(mem, s.as_str()).with_metadata(Metadata{ read_name: String::new(), location: token.location, documentation: String::new() });
                let y =
                if quoted {
                    let vec = vec![mem.symbol_for("quote"), x];
                    quoted = false;
                    vec_to_list(mem, &vec)
                }
                else {
                    x
                };
                if let Some((vec, _)) = stack.last_mut() {
                    vec.push(y);
                    continue;
                }
                else {
                    result = y;
                }
            },
        }

        if quoted {
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
            start_column = *x as usize - 1;
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
        location = Location::Stdin { line: 1, column: 0 };
    }

    let input = args[0].clone();

    match read_internal(mem, input, location) {
        Ok((result, rest)) => {
            let kv = vec![("status", mem.symbol_for("ok")), ("result", result), ("rest", rest.string), ("line", mem.allocate_number(rest.line as i64)), ("column", mem.allocate_number(rest.column as i64))];
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
            Ok(format_error(mem, location, msg, rest))
        },
    }
}


#[cfg(test)]
mod tests;

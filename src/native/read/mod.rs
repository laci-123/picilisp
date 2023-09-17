#![allow(dead_code)]

use crate::memory::*;
use crate::util::*;
use crate::native::list::make_plist;
use crate::error_utils::*;
use super::NativeFunctionMetaData;
use std::path::PathBuf;


enum TokenKind {
    Number(i64),
    Character(char),
    Symbol(String),
    OpenParen,
    CloseParen,
    LispString(String),
}

struct Token {
    token_kind: TokenKind,
    location: Location,
}

impl Token {
    fn new(token_kind: TokenKind, location: Location) -> Self {
        Self{ token_kind, location }
    }
}

type RestOfInput = GcRef;

enum TokenResult {
    InvalidString,
    Incomplete,
    Nothing,
    Error(Location, String, RestOfInput),
    Ok(Token, RestOfInput),
}

enum Source {
    Prelude,
    Stdin,
    File(PathBuf),
}

fn make_location(source: &Source, line: usize, column: usize) -> Location {
    match source {
        Source::Prelude    => Location::Prelude{ line, column },
        Source::Stdin      => Location::Stdin{ line, column },
        Source::File(path) => Location::File { path: path.clone(), line, column },
    }
}


fn next_token(input: GcRef, source: &Source, line_count: &mut usize, char_count: &mut usize) -> TokenResult {
    let mut cursor           = input;
    let mut next_cursor;
    let mut start_line_count = 1;
    let mut start_char_count = 0;
    let mut ch;
    let mut in_comment       = false;
    let mut in_string        = false;
    let mut string_escape    = false;
    let mut buffer           = String::new();

    while !cursor.is_nil() {
        if let Some((head, tail)) = next_character(cursor.clone()) {
            ch = head;
            next_cursor = tail;
            *char_count += 1;
        }
        else {
            return TokenResult::InvalidString;
        }

        match ch {
            '(' => {
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, source, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    return TokenResult::Ok(Token::new(TokenKind::OpenParen, make_location(source, *line_count, *char_count)), next_cursor);
                }
            },
            ')' => {
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, source, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    return TokenResult::Ok(Token::new(TokenKind::CloseParen, make_location(source, *line_count, *char_count)), next_cursor);
                }
            },
            '"' => {
                if in_string {
                    if string_escape {
                        buffer.push(ch);
                        string_escape = false;
                    }
                    else {
                        return string_token(&buffer, source, start_line_count, start_char_count, next_cursor);
                    }
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, source, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    in_string = true;
                    start_line_count = *line_count;
                    start_char_count = *char_count;
                }
            },
            '\n' => {
                *line_count += 1;
                *char_count = 0;
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if in_comment {
                    in_comment = false;
                }
                else if buffer.len() > 0 {
                    return atom_token(&buffer, source, start_line_count, start_char_count, next_cursor);
                }
                else {
                    // do nothing
                }
            }
            ';' => {
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, source, start_line_count, start_char_count, cursor);
                }
                else {
                    in_comment = true;
                }
            }
            c if c.is_whitespace() || c == ',' => {
                if in_string {
                    buffer.push(ch);
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, source, start_line_count, start_char_count, cursor);
                }
                else {
                    // do nothing
                }
            },
            c => {
                if in_string {
                    if c == '\\' && !string_escape {
                        string_escape = true;
                    }
                    else {
                        string_escape = false;
                    }
                    buffer.push(c);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    if buffer.len() == 0 {
                        start_line_count = *line_count;
                        start_char_count = *char_count;
                    }
                    buffer.push(c);
                }
            }
        }

        cursor = next_cursor;
    }

    if buffer.len() == 0 {
        return TokenResult::Nothing;
    }
    else if in_string {
        return TokenResult::Incomplete;
    }
    else {
        return atom_token(&buffer, source, start_line_count, start_char_count, cursor);
    }
}


fn string_token(string: &str, source: &Source, line_count: usize, char_count: usize, rest: RestOfInput) -> TokenResult {
    let mut escape = false;
    let mut result = String::new();
    let mut lc = 0;
    let mut cc = 0;

    for ch in string.chars() {
        cc += 1;
        if ch == '\\' {
            escape = true;
        }
        else if escape {
            match ch {
                '\\' => result.push('\\'),
                'n'  => result.push('\n'),
                'r'  => result.push('\r'),
                't'  => result.push('\t'),
                '"'  => result.push('"'),
                _    => {
                    let error_char_count =
                    if lc == 0 {
                        char_count + cc
                    }
                    else {
                        cc
                    };
                    return TokenResult::Error(make_location(source, line_count + lc, error_char_count), format!("'{ch}' is not a valid escape character in a string literal"), rest);
                },
            }
            escape = false;
        }
        else {
            if ch == '\n' {
                lc += 1;
                cc = 0;
            }
            result.push(ch);
        }
    }
    
    return TokenResult::Ok(Token::new(TokenKind::LispString(result), make_location(source, line_count, char_count)), rest);
}


fn atom_token(string: &str, source: &Source, line_count: usize, char_count: usize, rest: RestOfInput) -> TokenResult {
    if let Some(x) = number_token(string) {
        TokenResult::Ok(Token::new(TokenKind::Number(x), make_location(source, line_count, char_count)), rest)
    }
    else if let Some(x) = character_token(string) {
        TokenResult::Ok(Token::new(TokenKind::Character(x), make_location(source, line_count, char_count)), rest)
    }
    else if string.starts_with("#<") {
        TokenResult::Error(make_location(source, line_count, char_count), format!("unreadable symbol: {string}"), rest)
    }
    else {
        TokenResult::Ok(Token::new(TokenKind::Symbol(string.to_string()), make_location(source, line_count, char_count)), rest)
    }
}


fn number_token(string: &str) -> Option<i64> {
    string.parse().ok()
}


fn character_token(string: &str) -> Option<char> {
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
                Some(c3)
            }
            else {
                None
            }
        }
        else {
            Some(c1)
        }
    }
    else {
        None
    }
}


fn push_or_ok(mem: &mut Memory, stack: &mut Vec<Vec<GcRef>>, elem: GcRef, rest: GcRef, location: Location, rest_line: usize, rest_column: usize) -> Option<GcRef> {
    let mut read_name = "".to_string();
    if let Some(PrimitiveValue::Symbol(symbol)) = elem.get() {
        read_name = symbol.get_name();
    }

    let result = mem.allocate_metadata(elem, Metadata{ read_name, location, documentation: "".to_string(), parameters: vec![] });

    if let Some(top) = stack.last_mut() {
        top.push(result);
        None
    }
    else {
        let ok_sym = mem.symbol_for("ok");
        let ln     = mem.allocate_number(rest_line as i64);
        let cn     = mem.allocate_number((rest_column + 1) as i64);
        Some(make_plist(mem, &vec![("status", ok_sym), ("result", result), ("rest", rest.clone()), ("line", ln), ("column", cn)]))
    }
}


fn format_error(mem: &mut Memory, location: Location, msg: String, rest: RestOfInput, rest_line: usize, rest_column: usize) -> GcRef {
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


fn read_internal(mem: &mut Memory, input: GcRef, source: &Source, start_line_count: usize, start_char_count: usize) -> GcRef {
    let incomplete_sym  = mem.symbol_for("incomplete");
    let nothing_sym     = mem.symbol_for("nothing");
    let invalid_sym     = mem.symbol_for("invalid");
    let incomplete      = make_plist(mem, &vec![("status", incomplete_sym)]);
    let nothing         = make_plist(mem, &vec![("status", nothing_sym)]);
    let invalid         = make_plist(mem, &vec![("status", invalid_sym)]);

    let mut line_count = start_line_count;
    let mut char_count = start_char_count - 1;
    let mut cursor = input;
    let mut stack: Vec<Vec<GcRef>>  = vec![];

    loop {
        let (token, rest) =
        match next_token(cursor, &source, &mut line_count, &mut char_count) {
            TokenResult::Ok(t, rest)                => (t, rest),
            TokenResult::Error(location, msg, rest) => return format_error(mem, location, msg, rest, line_count + 1, char_count + 1),
            TokenResult::Incomplete                 => return incomplete,
            TokenResult::Nothing                    => return if stack.len() == 0 {nothing} else {incomplete},
            TokenResult::InvalidString              => return invalid,
        };

        match token.token_kind {
            TokenKind::Number(x) => {
                let y = mem.allocate_number(x);
                if let Some(z) = push_or_ok(mem, &mut stack, y, rest.clone(), token.location, line_count, char_count) {return z;}
            },
            TokenKind::Character(x) => {
                let y = mem.allocate_character(x);
                if let Some(z) = push_or_ok(mem, &mut stack, y, rest.clone(), token.location, line_count, char_count) {return z;}
            },
            TokenKind::Symbol(x) => {
                let y = mem.symbol_for(&x);
                if let Some(z) = push_or_ok(mem, &mut stack, y, rest.clone(), token.location, line_count, char_count) {return z;}
            },
            TokenKind::LispString(x) => {
                let y = string_to_proper_list(mem, &x);
                if let Some(z) = push_or_ok(mem, &mut stack, y, rest.clone(), token.location, line_count, char_count) {return z;}
            },
            TokenKind::OpenParen => {
                stack.push(vec![]);
            },
            TokenKind::CloseParen => {
                if let Some(vec) = stack.pop() {
                    let y = vec_to_list(mem, &vec);
                    if let Some(z) = push_or_ok(mem, &mut stack, y, rest.clone(), token.location, line_count, char_count) {return z;}
                }
                else {
                    return format_error(mem, make_location(source, line_count, char_count), "too many closing parentheses".to_string(), rest, line_count + 1, char_count + 1);
                }
            },
        }

        cursor = rest;
    }
}


fn next_character(input: GcRef) -> Option<(char, GcRef)> {
    let cons =
    if let Some(PrimitiveValue::Cons(cons)) = input.get() {
        cons
    }
    else {
        return None;
    };

    if let Some(PrimitiveValue::Character(ch)) = cons.get_car().get() {
        Some((*ch, cons.get_cdr()))
    }
    else {
        None
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
    if args.len() != 1 && args.len() != 4 {
        let vec           = vec![mem.symbol_for("or"), mem.allocate_number(1), mem.allocate_number(4)];
        let error_details = vec![("expected", vec_to_list(mem, &vec)), ("actual", fit_to_number(mem, args.len()))];
        let error         = make_error(mem, "wrong-number-of-arguments", READ.name, &error_details);
        return Err(error);
    }

    let source;
    let start_line;
    let start_column;
    if args.len() == 4 {
        if let Some(path) = list_to_string(args[1].clone()) {
            source = Source::File(PathBuf::from(path));
        }
        else if symbol_eq!(args[1], mem.symbol_for("prelude")) {
                source = Source::Prelude;
        }
        else if symbol_eq!(args[1], mem.symbol_for("stdin")) {
                source = Source::Stdin;
        }
        else {
            let error_details = vec![("the-unknown-source", args[1].clone())];
            let error = make_error(mem, "unknown-read-source", READ.name, &error_details);
            return Err(error);
        }

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
    }
    else {
        source = Source::Stdin;
        start_line = 1;
        start_column = 1;
    }

    Ok(read_internal(mem, args[0].clone(), &source, start_line, start_column))
}


#[cfg(test)]
mod tests;

#![allow(dead_code)]

use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use std::path::Path;


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


fn next_token(input: GcRef, file: Option<&Path>, line_count: &mut usize, char_count: &mut usize) -> TokenResult {
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
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    return TokenResult::Ok(Token::new(TokenKind::OpenParen, Location::new(file, *line_count, *char_count)), next_cursor);
                }
            },
            ')' => {
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    return TokenResult::Ok(Token::new(TokenKind::CloseParen, Location::new(file, *line_count, *char_count)), next_cursor);
                }
            },
            '"' => {
                if in_string {
                    if string_escape {
                        buffer.push(ch);
                        string_escape = false;
                    }
                    else {
                        return string_token(&buffer, file, start_line_count, start_char_count, next_cursor);
                    }
                }
                else if buffer.len() > 0 {
                    *char_count -= 1;
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
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
                    return atom_token(&buffer, file, start_line_count, start_char_count, next_cursor);
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
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
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
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
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
        return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
    }
}


fn string_token(string: &str, file: Option<&Path>, line_count: usize, char_count: usize, rest: RestOfInput) -> TokenResult {
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
                    return TokenResult::Error(Location::new(file, line_count + lc, error_char_count), format!("'{ch}' is not a valid escape character in a string literal"), rest);
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
    
    return TokenResult::Ok(Token::new(TokenKind::LispString(result), Location::new(file, line_count, char_count)), rest);
}


fn atom_token(string: &str, file: Option<&Path>, line_count: usize, char_count: usize, rest: RestOfInput) -> TokenResult {
    if let Some(x) = number_token(string) {
        TokenResult::Ok(Token::new(TokenKind::Number(x), Location::new(file, line_count, char_count)), rest)
    }
    else if let Some(x) = character_token(string) {
        TokenResult::Ok(Token::new(TokenKind::Character(x), Location::new(file, line_count, char_count)), rest)
    }
    else {
        TokenResult::Ok(Token::new(TokenKind::Symbol(string.to_string()), Location::new(file, line_count, char_count)), rest)
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
    let meta = mem.allocate_metadata(elem, Metadata{ location, documentation: "".to_string() });
    if let Some(top) = stack.last_mut() {
        top.push(meta);
        None
    }
    else {
        let ok_sym = mem.symbol_for("ok");
        let ln     = mem.allocate_number(rest_line as i64);
        let cn     = mem.allocate_number((rest_column + 1) as i64);
        let return_value = vec_to_list(mem, &vec![ok_sym, meta, rest.clone(), ln, cn]);
        Some(return_value)
    }
}


fn format_error(mem: &mut Memory, location: Location, msg: String, rest: RestOfInput, rest_line: usize, rest_column: usize) -> GcRef {
    let error_sym = mem.symbol_for("error");
    let error_msg = string_to_proper_list(mem, &msg);
    let file      = if let Some(f) = location.file {
        string_to_proper_list(mem, &f.into_os_string().into_string().unwrap())
    }
    else {
        mem.symbol_for("stdin")
    };
    let line      = mem.allocate_number(location.line as i64);
    let column    = mem.allocate_number(location.column as i64);
    let error_loc = vec_to_list(mem, &vec![file, line, column]);
    let error     = mem.allocate_cons(error_loc, error_msg);
    let ln        = mem.allocate_number(rest_line as i64);
    let cn        = mem.allocate_number(rest_column as i64);
    vec_to_list(mem, &vec![error_sym, error, rest, ln, cn])
}


fn read_internal(mem: &mut Memory, input: GcRef, file: Option<&Path>, start_line_count: usize, start_char_count: usize) -> GcRef {
    let incomplete_sym  = mem.symbol_for("incomplete");
    let nothing_sym     = mem.symbol_for("nothing");
    let invalid_sym     = mem.symbol_for("invalid");
                                              // status          result        rest          line          column
    let incomplete      = vec_to_list(mem, &vec![incomplete_sym, GcRef::nil(), GcRef::nil(), GcRef::nil(), GcRef::nil()]);
    let nothing         = vec_to_list(mem, &vec![nothing_sym,    GcRef::nil(), GcRef::nil(), GcRef::nil(), GcRef::nil()]);
    let invalid         = vec_to_list(mem, &vec![invalid_sym,    GcRef::nil(), GcRef::nil(), GcRef::nil(), GcRef::nil()]);

    let mut line_count = start_line_count;
    let mut char_count = start_char_count - 1;
    let mut cursor = input;
    let mut stack: Vec<Vec<GcRef>>  = vec![];

    loop {
        let (token, rest) =
        match next_token(cursor, file, &mut line_count, &mut char_count) {
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
                    return format_error(mem, Location::new(file, line_count, char_count), "too many closing parentheses".to_string(), rest, line_count + 1, char_count + 1);
                }
            },
        }

        cursor = rest;
    }
}


fn next_character(input: GcRef) -> Option<(char, GcRef)> {
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


/// Converts a Lisp-style string to an AST
///
/// Only reads the shortest prefix of `input` that is a valid AST
///
/// Returns: `(list status result rest line column)`
/// where `status` can be one of the following:
///  * `ok`:         Success. `result` is the AST.
///  * `nothing`:    `input` was empty or only contained whitespace. `result` is undefined.
///  * `incomplete`: `input` is not a valid AST, but can be the beginning of a valid AST. `result` is undefined.
///  * `error`:      `input` is not a valid AST, not even the beginning of one. `result` contains the error details in `(cons error-location error-message)` format.
///  * `invalid`:    `input` is not a valid string. `result`, `rest`, `line` and `column` are undefined.
///  * `line`:       The starting line number of `rest`
///  * `column`:     The starting column number of `rest`
/// `result` is the read AST and
/// `rest` is the unread rest of `input` (`nil` if all of `input` was read).
pub fn read(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 && args.len() != 3 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    let start_line;
    let start_column;
    if args.len() == 3 {
        if let PrimitiveValue::Number(x) = args[1].get() {
            start_line = *x as usize;
        }
        else {
            return NativeResult::Signal(mem.symbol_for("wrong-arg-type"));
        }
        if let PrimitiveValue::Number(x) = args[2].get() {
            start_column = *x as usize;
        }
        else {
            return NativeResult::Signal(mem.symbol_for("wrong-arg-type"));
        }
    }
    else {
        start_line = 1;
        start_column = 1;
    }

    NativeResult::Value(read_internal(mem, args[0].clone(), None, start_line, start_column))
}


#[cfg(test)]
mod tests;

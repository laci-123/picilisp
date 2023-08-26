#![allow(dead_code)]

use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use std::path::PathBuf;


enum TokenKind {
    Number(f64),
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


fn fetch_token(input: GcRef, file: Option<PathBuf>) -> TokenResult {
    let mut cursor           = input;
    let mut next_cursor;
    let mut line_count       = 0;
    let mut char_count       = 0;
    let mut start_line_count = 0;
    let mut start_char_count = 0;
    let mut ch;
    let mut in_comment       = false;
    let mut in_string        = false;
    let mut string_escape    = false;
    let mut buffer           = String::new();

    while !cursor.is_nil() {
        if let Some((head, tail)) = fetch_character(cursor.clone()) {
            ch = head;
            next_cursor = tail;
        }
        else {
            return TokenResult::InvalidString;
        }

        char_count += 1;
        match ch {
            '(' => {
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if buffer.len() > 0 {
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    return TokenResult::Ok(Token::new(TokenKind::OpenParen, Location::new(file, line_count, char_count)), next_cursor);
                }
            },
            ')' => {
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if buffer.len() > 0 {
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    return TokenResult::Ok(Token::new(TokenKind::CloseParen, Location::new(file, line_count, char_count)), next_cursor);
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
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
                }
                else if in_comment {
                    // do nothing
                }
                else {
                    in_string = true;
                }
            },
            '\n' => {
                line_count += 1;
                char_count = 0;
                if in_string {
                    buffer.push(ch);
                    string_escape = false;
                }
                else if in_comment {
                    in_comment = false;
                }
                else if buffer.len() > 0 {
                    return atom_token(&buffer, file, start_line_count, start_char_count, cursor);
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
                else {
                    in_comment = true;
                }
            }
            c if c.is_whitespace() => {
                if in_string {
                    buffer.push(ch);
                }
                else if buffer.len() > 0 {
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
                        start_line_count = line_count;
                        start_char_count = char_count;
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


fn string_token(string: &str, file: Option<PathBuf>, line_count: usize, char_count: usize, rest: RestOfInput) -> TokenResult {
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


fn atom_token(string: &str, file: Option<PathBuf>, line_count: usize, char_count: usize, rest: RestOfInput) -> TokenResult {
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


fn number_token(string: &str) -> Option<f64> {
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


fn read_internal(mem: &mut Memory, input: GcRef) -> GcRef {
    let ok_sym          = mem.symbol_for("ok");
    let incomplete_sym  = mem.symbol_for("incomplete");
    let nothing_sym     = mem.symbol_for("nothing");
    let invalid_sym     = mem.symbol_for("invalid");
    let error_sym       = mem.symbol_for("error");
    let incomplete      = vec_to_list(mem, &vec![incomplete_sym, GcRef::nil(), GcRef::nil()]);
    let nothing         = vec_to_list(mem, &vec![nothing_sym, GcRef::nil(), GcRef::nil()]);
    let invalid         = vec_to_list(mem, &vec![invalid_sym, GcRef::nil(), GcRef::nil()]);

    let mut cursor = input;
    let mut stack: Vec<Vec<GcRef>>  = vec![];

    loop {
        let (token, rest) =
        match fetch_token(cursor, None) {
            TokenResult::Ok(t, rest)         => (t, rest),
            TokenResult::Error(_, msg, rest) => {
                let error_msg = string_to_proper_list(mem, &msg);
                return vec_to_list(mem, &vec![error_sym, error_msg, rest])
            },
            TokenResult::Incomplete          => return incomplete,
            TokenResult::Nothing             => return if stack.len() == 0 {nothing} else {incomplete},
            TokenResult::InvalidString       => return invalid,
        };

        let push_or_ok = |mem: &mut Memory, stack: &mut Vec<Vec<GcRef>>, elem: GcRef| {
            if let Some(top) = stack.last_mut() {
                top.push(elem);
                None
            }
            else {
                let return_value = vec_to_list(mem, &vec![ok_sym.clone(), elem, rest.clone()]);
                Some(return_value)
            }
        };

        match token.token_kind {
            TokenKind::Number(x) => {
                let y = mem.allocate_number(x);
                if let Some(z) = push_or_ok(mem, &mut stack, y) {return z;}
            },
            TokenKind::Character(x) => {
                let y = mem.allocate_character(x);
                if let Some(z) = push_or_ok(mem, &mut stack, y) {return z;}
            },
            TokenKind::Symbol(x) => {
                let y = mem.symbol_for(&x);
                if let Some(z) = push_or_ok(mem, &mut stack, y) {return z;}
            },
            TokenKind::LispString(x) => {
                let y = string_to_proper_list(mem, &x);
                if let Some(z) = push_or_ok(mem, &mut stack, y) {return z;}
            },
            TokenKind::OpenParen => {
                stack.push(vec![]);
            },
            TokenKind::CloseParen => {
                if let Some(vec) = stack.pop() {
                    let y = vec_to_list(mem, &vec);
                    if let Some(z) = push_or_ok(mem, &mut stack, y) {return z;}
                }
                else {
                    let error_msg = string_to_proper_list(mem, "too many closing parentheses");
                    return vec_to_list(mem, &vec![error_sym, error_msg, rest])
                }
            },
        }

        cursor = rest;
    }
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


/// Converts a Lisp-style string to an AST
///
/// Only reads the shortest prefix of `input` that is a valid AST
///
/// Returns: `(list status result rest)`
/// where `status` can be one of the following:
///  * `ok`:         Success. `result` is the AST.
///  * `nothing`:    `input` was empty or only contained whitespace. `result` is undefined.
///  * `incomplete`: `input` is not a valid AST, but can be the beginning of a valid AST. `result` is undefined.
///  * `error`:      `input` is not a valid AST, not even the beginning of one. `result` contains the error details.
///  * `invalid`:    `input` is not a valid string. `result` and `rest` are undefined.
/// `result` is the read AST and
/// `rest` is the unread rest of `input`.
pub fn read(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    NativeResult::Value(read_internal(mem, args[0].clone()))
}


#[cfg(test)]
mod tests;

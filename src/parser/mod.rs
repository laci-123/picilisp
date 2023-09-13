#![allow(dead_code)]

use crate::{memory::*, util::vec_to_list};



#[derive(Clone)]
pub struct StringWithLocation {
    pub string: GcRef,
    pub location: Location,
}


#[derive(Clone)]
pub struct ParserOk {
    pub value: GcRef,
    pub location: Location,
    pub rest_of_input: StringWithLocation,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ParserErrorKind {
    Fatal,
    Unfinished,
}

use ParserErrorKind::*;

fn the_worse_one(one: ParserErrorKind, other: ParserErrorKind) -> ParserErrorKind {
    if let Fatal = one {
        Fatal
    }
    else if let Fatal = other {
        Fatal
    }
    else {
        Unfinished
    }
}


#[derive(Clone)]
pub struct ParserError {
    pub expected: String,
    pub error_loc: Location,
    pub kind: ParserErrorKind,
}


pub trait Parser: Fn(&mut Memory, StringWithLocation) -> Result<ParserOk, ParserError> {}

impl<C: Fn(&mut Memory, StringWithLocation) -> Result<ParserOk, ParserError>> Parser for C {}


pub fn satisfy<'a>(parser: &'a impl Parser, predicate: &'a impl Fn(GcRef) -> bool, expected: &'a str) -> impl Parser + 'a {
    |mem, input| {
        let output = parser(mem, input)?;
        if predicate(output.value.clone()) {
            Ok(output)
        }
        else {
            Err(ParserError{ expected: expected.to_string(), error_loc: output.location, kind: Fatal })
        }
    }
}


pub fn one_of_these_two<'a>(parser1: &'a impl Parser, parser2: &'a impl Parser) -> impl Parser + 'a {
    |mem, input| {
        match parser1(mem, input.clone()) {
            Ok(output1) => Ok(output1),
            Err(err1)   => {
                match parser2(mem, input.clone()) {
                    Ok(output2) => Ok(output2),
                    Err(err2)   => Err(ParserError{ expected: format!("either ({}) or ({})", err1.expected, err2.expected), error_loc: input.location, kind: the_worse_one(err1.kind, err2.kind) })
                }
            },
        }
    }
}


pub fn one_of_these_three<'a>(parser1: &'a impl Parser, parser2: &'a impl Parser, parser3: &'a impl Parser) -> impl Parser + 'a {
    |mem, input| {
        match parser1(mem, input.clone()) {
            Ok(output1) => Ok(output1),
            Err(err1)   => {
                match parser2(mem, input.clone()) {
                    Ok(output2) => Ok(output2),
                    Err(err2)   => {
                        match parser3(mem, input.clone()) {
                            Ok(output3) => Ok(output3),
                            Err(err3)   => Err(ParserError{ expected:  format!("either ({}), ({}) or ({})", err1.expected, err2.expected, err3.expected),
                                                            error_loc: input.location,
                                                            kind: the_worse_one(err1.kind, the_worse_one(err1.kind, err2.kind)) })
                        }
                    }
                }
            },
        }
    }
}


pub fn zero_or_more_times<'a>(parser: &'a impl Parser) -> impl Parser + 'a {
    |mem, input| {
        let mut outputs     = vec![];
        let mut cursor      = input.clone();
        let mut prev_cursor = input.clone();

        loop {
            match parser(mem, cursor) {
                Ok(output) => {
                    outputs.push(output.value);
                    cursor = output.rest_of_input;
                    prev_cursor = cursor.clone();
                },
                Err(_) => break,
            }
        }

        Ok(ParserOk{ value: vec_to_list(mem, &outputs), location: input.location, rest_of_input: prev_cursor })
    }
}


pub fn at_least_once<'a>(parser: &'a impl Parser) -> impl Parser + 'a {
    |mem, input| {
        match parser(mem, input) {
            Ok(output) => {
                let tail = (zero_or_more_times(parser))(mem, output.rest_of_input)?;
                let list = mem.allocate_cons(output.value, tail.value);
                Ok(ParserOk{ value: list, location: output.location, rest_of_input: tail.rest_of_input })
            },
            Err(err)   => Err(err),
        }
    }
}


pub fn parse_or_default<'a>(parser: &'a impl Parser, default: &'a GcRef) -> impl Parser + 'a {
    |mem, input| {
        match parser(mem, input.clone()) {
            Ok(output) => Ok(output),
            Err(_)     => Ok(ParserOk { value: default.clone(), location: input.location.clone(), rest_of_input: input })
        }
    }
}


pub fn any_character(mem: &mut Memory, input: StringWithLocation) -> Result<ParserOk, ParserError> {
    match input.string.get() {
        Some(PrimitiveValue::Cons(cons)) => {
            if let Some(PrimitiveValue::Character(ch)) = cons.get_car().get() {
                let this_location = input.location.clone();
                let rest_string = cons.get_cdr();
                let rest_location =
                if *ch == '\n' {
                    input.location.step_line()
                }
                else {
                    input.location.step_column()
                };
                Ok(ParserOk{ value: mem.allocate_character(*ch), location: this_location, rest_of_input: StringWithLocation{ string: rest_string, location: rest_location } })
            }
            else {
                Err(ParserError{ expected: "a character".to_string(), error_loc: input.location, kind: Fatal })
            }
        },
        Some(_) => {
            Err(ParserError{ expected: "a character".to_string(), error_loc: input.location, kind: Fatal })
        },
        None    => {
            Err(ParserError{ expected: "a character".to_string(), error_loc: input.location, kind: Unfinished })
        },
    }
}

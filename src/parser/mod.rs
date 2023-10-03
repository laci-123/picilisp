#![allow(dead_code)]

use crate::{memory::*, util::vec_to_list};
use crate::metadata::*;
use crate::config;



#[derive(Clone)]
pub struct StringWithLocation {
    pub string: GcRef,
    pub location: Location,
}

impl StringWithLocation {
    pub fn next(self) -> Self {
        if let Some(PrimitiveValue::Cons(cons)) = self.string.get() {
            if let Some(PrimitiveValue::Character(ch)) = cons.get_car().get() {
                if *ch == '\n' {
                    return Self{ string: cons.get_cdr(), location: self.location.step_line() };
                }
                else {
                    return Self{ string: cons.get_cdr(), location: self.location.step_column() };
                }
            }
        }

        self
    }
    
    pub fn trim(self) -> Self {
        if let Some(PrimitiveValue::Cons(cons)) = self.string.get() {
            if let Some(PrimitiveValue::Character(ch)) = cons.get_car().get() {
                if *ch == '\n' {
                    return Self{ string: cons.get_cdr(), location: self.location.step_line() };
                }
            }
        }

        self
    }
}

#[derive(Clone)]
pub struct ParserOk {
    pub value: GcRef,
    pub location: Location,
    pub rest_of_input: StringWithLocation,
}


#[derive(Clone)]
pub enum ParserError {
    NoMatch,
    Incomplete,
    Fatal{message: String, location: Location, rest_of_input: StringWithLocation},
    StackOverflow,
}


pub fn check_stack_overflow(recursion_depth: usize) -> Result<ParserOk, ParserError> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        Err(ParserError::StackOverflow)
    }
    else {
        Ok(ParserOk { value: GcRef::nil(), location: Location::Native, rest_of_input: StringWithLocation{ string: GcRef::nil(), location: Location::Native } })
    }
}


fn select_error(e1: ParserError, e2: ParserError) -> ParserError {
    use ParserError::*;
    
    match e1 {
        NoMatch    => NoMatch,
        Incomplete => match e2 {
            NoMatch       => NoMatch,
            Incomplete    => Incomplete,
            StackOverflow => StackOverflow,
            _             => e1,
        }
        _          => e1,
    }
}


pub trait Parser: Fn(&mut Memory, StringWithLocation, usize) -> Result<ParserOk, ParserError> {}

impl<C: Fn(&mut Memory, StringWithLocation, usize) -> Result<ParserOk, ParserError>> Parser for C {}


pub fn satisfy<'a>(parser: &'a impl Parser, predicate: &'a impl Fn(GcRef) -> bool) -> impl Parser + 'a {
    |mem, input, recursion_depth| {
        check_stack_overflow(recursion_depth)?;

        let output = parser(mem, input, recursion_depth + 1)?;
        if predicate(output.value.clone()) {
            Ok(output)
        }
        else {
            Err(ParserError::NoMatch)
        }
    }
}


pub fn one_of_these_two<'a>(parser1: &'a impl Parser, parser2: &'a impl Parser) -> impl Parser + 'a {
    |mem, input, recursion_depth| {
        check_stack_overflow(recursion_depth)?;

        match parser1(mem, input.clone(), recursion_depth + 1) {
            Ok(output1) => Ok(output1),
            Err(ParserError::StackOverflow) => Err(ParserError::StackOverflow),
            Err(err1)   => {
                match parser2(mem, input.clone(), recursion_depth + 1) {
                    Ok(output2) => Ok(output2),
                    Err(ParserError::StackOverflow) => Err(ParserError::StackOverflow),
                    Err(err2)   => Err(select_error(err1, err2)),
                }
            },
        }
    }
}


pub fn one_of_these_three<'a>(parser1: &'a impl Parser, parser2: &'a impl Parser, parser3: &'a impl Parser) -> impl Parser + 'a {
    |mem, input, recursion_depth| {
        check_stack_overflow(recursion_depth)?;

        match parser1(mem, input.clone(), recursion_depth + 1) {
            Ok(output1) => Ok(output1),
            Err(ParserError::StackOverflow) => Err(ParserError::StackOverflow),
            Err(err1)   => {
                match parser2(mem, input.clone(), recursion_depth + 1) {
                    Ok(output2) => Ok(output2),
                    Err(ParserError::StackOverflow) => Err(ParserError::StackOverflow),
                    Err(err2)   => {
                        match parser3(mem, input.clone(), recursion_depth + 1) {
                            Ok(output3) => Ok(output3),
                            Err(ParserError::StackOverflow) => Err(ParserError::StackOverflow),
                            Err(err3)   => Err(select_error(err1, select_error(err2, err3))),
                        }
                    }
                }
            },
        }
    }
}


pub fn zero_or_more_times<'a>(parser: &'a impl Parser) -> impl Parser + 'a {
    |mem, input, recursion_depth| {
        check_stack_overflow(recursion_depth)?;

        let mut outputs     = vec![];
        let mut cursor      = input.clone();
        let mut prev_cursor = input.clone();

        loop {
            match parser(mem, cursor, recursion_depth + 1) {
                Ok(output) => {
                    outputs.push(output.value);
                    cursor = output.rest_of_input;
                    prev_cursor = cursor.clone();
                },
                Err(ParserError::StackOverflow) => return Err(ParserError::StackOverflow),
                Err(ParserError::Fatal { message, location, rest_of_input }) => return Err(ParserError::Fatal { message, location, rest_of_input }),
                Err(_) => break,
            }
        }

        Ok(ParserOk{ value: vec_to_list(mem, &outputs), location: input.location, rest_of_input: prev_cursor })
    }
}


pub fn at_least_once<'a>(parser: &'a impl Parser) -> impl Parser + 'a {
    |mem, input, recursion_depth| {
        check_stack_overflow(recursion_depth)?;

        match parser(mem, input, recursion_depth + 1) {
            Ok(output) => {
                let tail = (zero_or_more_times(parser))(mem, output.rest_of_input, recursion_depth + 1)?;
                let list = mem.allocate_cons(output.value, tail.value);
                Ok(ParserOk{ value: list, location: output.location, rest_of_input: tail.rest_of_input })
            },
            Err(err)   => Err(err),
        }
    }
}


pub fn parse_or_default<'a>(parser: &'a impl Parser, default: &'a GcRef) -> impl Parser + 'a {
    |mem, input, recursion_depth| {
        check_stack_overflow(recursion_depth)?;

        match parser(mem, input.clone(), recursion_depth + 1) {
            Ok(output) => Ok(output),
            Err(ParserError::StackOverflow) => Err(ParserError::StackOverflow),
            Err(_)     => Ok(ParserOk { value: default.clone(), location: input.location.clone(), rest_of_input: input })
        }
    }
}


pub fn end_of_input(_mem: &mut Memory, input: StringWithLocation, recursion_depth: usize) -> Result<ParserOk, ParserError> {
    check_stack_overflow(recursion_depth)?;

    if input.string.is_nil() {
        Ok(ParserOk{ value: GcRef::nil(), location: input.location.clone(), rest_of_input: input })
    }
    else {
        Err(ParserError::NoMatch)
    } 
}


pub fn any_character(mem: &mut Memory, input: StringWithLocation, recursion_depth: usize) -> Result<ParserOk, ParserError> {
    check_stack_overflow(recursion_depth)?;

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
                Err(ParserError::NoMatch)
            }
        },
        Some(_) => {
            Err(ParserError::NoMatch)
        },
        None    => {
            Err(ParserError::Incomplete)
        },
    }
}

use crate::memory::*;
use crate::util::{string_to_list, list_to_string};
use crate::native::read::read;
use crate::native::eval::eval;
use crate::native::print::print;
use std::io::{self, BufRead};
use std::io::prelude::*;


pub fn repl(mem: &mut Memory, _args: &[GcRef], env: GcRef) -> NativeResult {
    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");

    let mut incomplete = false;
    let mut input      = String::new();

    let stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    for line in stdin.lock().lines() {
        input.push_str(&line.unwrap());
        input.push_str("\n"); // put back the newline to know where line comments end
        
        let input_list  = string_to_list(mem, input.as_str());
        let read_output = 
        match read(mem, &[input_list], env.clone()) {
            NativeResult::Value(x) => x,
            other                  => return other,
        };

        let cons1  = read_output.get().as_conscell();
        let car1   = cons1.get_car();
        let status = car1.get().as_symbol();
        let cons2  = cons1.get_cdr();
        let result = cons2.get().as_conscell().get_car();
        // ignore rest of input

        if status == ok_symbol.get().as_symbol() {
            incomplete = false;
            input.clear();

            let ast    = result;
            let evaled =
            match eval(mem, &[ast], env.clone()) {
                NativeResult::Value(x) => x,
                other                  => return other,
            };

            let output =
            match print(mem, &[evaled], env.clone()) {
                NativeResult::Value(x) => x,
                other                  => return other,
            };

            println!("{}", list_to_string(output).unwrap());
        }
        else if status == incomplete_symbol.get().as_symbol() {
            incomplete = true;
        }
        else if status == error_symbol.get().as_symbol() {
            println!("SYNTAX ERROR");
            input.clear();
        }
        else if status == invalid_symbol.get().as_symbol() {
            return NativeResult::Signal(mem.symbol_for("invalid-string"));
        }
        else {
            unreachable!();
        }
        
        print!("{}", if incomplete {"... "} else {">>> "});
        io::stdout().flush().unwrap();
    }

    println!();

    NativeResult::Value(ok_symbol)
}

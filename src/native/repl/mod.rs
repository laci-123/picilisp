use crate::memory::*;
use crate::util::*;
use crate::native::read::read;
use crate::native::eval::eval;
use crate::native::print::print;
use crate::native::list::property;
use std::io::{self, BufRead};
use std::io::prelude::*;


pub fn repl(mem: &mut Memory, _args: &[GcRef], env: GcRef) -> NativeResult {
    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let nothing_symbol    = mem.symbol_for("nothing");
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
        let output = 
        match read(mem, &[input_list], env.clone()) {
            NativeResult::Value(x) => x,
            other                  => return other,
        };

        let status_tmp = property(mem, "status", output.clone()).unwrap();
        let status     = status_tmp.get().unwrap().as_symbol();

        if status == ok_symbol.get().unwrap().as_symbol() {
            incomplete = false;
            input.clear();

            let ast    = property(mem, "result", output.clone()).unwrap();
            let evaled =
            match eval(mem, &[ast], env.clone()) {
                NativeResult::Value(x)       => x,
                NativeResult::Signal(signal) => {
                    println!("UNHANDLED-SIGNAL:");
                    signal
                },
                NativeResult::Abort(msg)     => return NativeResult::Abort(msg),
            };

            let output =
            match print(mem, &[evaled], env.clone()) {
                NativeResult::Value(x) => x,
                other                  => return other,
            };

            println!("{}", list_to_string(output).unwrap());
        }
        else if status == incomplete_symbol.get().unwrap().as_symbol() {
            incomplete = true;
        }
        else if status == error_symbol.get().unwrap().as_symbol() {
            let error          = property(mem, "error", output.clone()).unwrap();
            let error_location = list_to_vec(error.get().unwrap().as_conscell().get_car()).unwrap();
            let line           = *error_location[1].get().unwrap().as_number();
            let column         = *error_location[2].get().unwrap().as_number();
            let error_message  = list_to_string(error.get().unwrap().as_conscell().get_cdr()).unwrap();
            println!("SYNTAX-ERROR: {error_message}");
            println!("       at <stdin>:{line}:{column}");
            input.clear();
        }
        else if status == invalid_symbol.get().unwrap().as_symbol() {
            return NativeResult::Signal(mem.symbol_for("invalid-string"));
        }
        else if status == nothing_symbol.get().unwrap().as_symbol() {
            // do nothing
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

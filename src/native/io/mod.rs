use crate::memory::*;
use crate::util::{list_to_string, string_to_list};
use crate::native::read::read;
use crate::native::eval::eval;
use std::fs::File;
use std::io::{prelude::*, BufReader};



pub fn message(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-number-of-arguments"));
    }
    
    if let Some(msg) = list_to_string(args[0].clone()) {
        println!("{msg}");
    }
    else {
        return NativeResult::Signal(mem.symbol_for("invalid-string"));
    }

    NativeResult::Value(mem.symbol_for("ok"))
}



pub fn load(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-number-of-arguments"));
    }

    let filename =
    if let Some(name) = list_to_string(args[0].clone()) {
        name
    }
    else {
        return NativeResult::Signal(mem.symbol_for("invalid-string"));
    };

    let file =
    if let Ok(f) = File::open(filename) {
        f
    }
    else {
        return NativeResult::Signal(mem.symbol_for("cannot-open-file"));
    };
    let reader = BufReader::new(file);


    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");

    let mut input = String::new();

    for line in reader.lines() {
        input.push_str(&line.unwrap());
        input.push_str("\n"); // put back the newline to know where line comments end
        
        let input_list  = string_to_list(mem, input.as_str());
        let read_output = 
        match read(mem, &[input_list], env.clone()) {
            NativeResult::Value(x) => x,
            other                  => return other,
        };

        let cons1  = read_output.get().unwrap().as_conscell();
        let car1   = cons1.get_car();
        let status = car1.get().unwrap().as_symbol();
        let cons2  = cons1.get_cdr();
        let result = cons2.get().unwrap().as_conscell().get_car();
        // ignore rest of input

        if status == ok_symbol.get().unwrap().as_symbol() {
            input.clear();

            let ast = result;
            match eval(mem, &[ast], env.clone()) {
                NativeResult::Value(_) => {/* only using the side effects */},
                other                  => return other,
            };

        }
        else if status == incomplete_symbol.get().unwrap().as_symbol() {
            // just wait
        }
        else if status == error_symbol.get().unwrap().as_symbol() {
            println!("SYNTAX ERROR");
            input.clear();
        }
        else if status == invalid_symbol.get().unwrap().as_symbol() {
            return NativeResult::Signal(mem.symbol_for("invalid-string"));
        }
        else {
            unreachable!();
        }
    }

    NativeResult::Value(ok_symbol)
}

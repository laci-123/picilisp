use crate::memory::*;
use crate::util::*;
use crate::native::read::read;
use crate::native::eval::eval;
use crate::native::print::print;
use crate::native::list::property;
use super::NativeFunctionMetaData;
use std::io::{self, BufRead};
use std::io::prelude::*;



pub const REPL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      repl,
    name:          "repl",
    kind:          FunctionKind::Lambda,
    parameters:    &[],
    documentation: "(R)ead an expression from standard input,
(E)valuated it,
(P)rint the result to standard output,
then repeat (or (L)oop) from the beginning.
Stop the loop when end of input (EOF) is reached.",
};

pub fn repl(mem: &mut Memory, _args: &[GcRef], env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let nothing_symbol    = mem.symbol_for("nothing");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");
    let stdin_symbol      = mem.symbol_for("stdin");
    let start_line        = mem.allocate_number(1);
    let start_column      = mem.allocate_number(1);

    let mut incomplete = false;
    let mut input      = String::new();

    let stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    for line in stdin.lock().lines() {
        input.push_str(&line.unwrap());
        input.push_str("\n"); // put back the newline to know where line comments end
        
        let input_list  = string_to_list(mem, input.as_str());
        let output = read(mem, &[input_list, stdin_symbol.clone(), start_line.clone(), start_column.clone()], env.clone(), recursion_depth + 1)?;

        let status = property(mem, "status", output.clone()).unwrap();

        if symbol_eq!(status, ok_symbol) {
            incomplete = false;
            input.clear();

            let ast    = property(mem, "result", output.clone()).unwrap();
            let evaled =
            match eval(mem, &[ast], env.clone(), recursion_depth + 1) {
                Ok(x)       => x,
                Err(signal) => {
                    println!("UNHANDLED-SIGNAL:");
                    signal
                },
            };

            let output = print(mem, &[evaled], env.clone(), recursion_depth + 1)?;

            println!("{}", list_to_string(output).unwrap());
        }
        else if symbol_eq!(status, incomplete_symbol) {
            incomplete = true;
        }
        else if symbol_eq!(status, error_symbol) {
            let error          = property(mem, "error", output.clone()).unwrap();
            let error_location = list_to_vec(property(mem, "location", error.clone()).unwrap()).unwrap();
            let line           = *error_location[1].get().unwrap().as_number();
            let column         = *error_location[2].get().unwrap().as_number();
            let error_message  = list_to_string(property(mem, "message", error).unwrap()).unwrap();
            println!("SYNTAX-ERROR: {error_message}");
            println!("       at <stdin>:{line}:{column}");
            input.clear();
        }
        else if symbol_eq!(status, invalid_symbol) {
            return Err(mem.symbol_for("invalid-string"));
        }
        else if symbol_eq!(status, nothing_symbol) {
            // do nothing
        }
        else {
            unreachable!();
        }
        
        print!("{}", if incomplete {"... "} else {">>> "});
        io::stdout().flush().unwrap();
    }

    println!();

    Ok(ok_symbol)
}

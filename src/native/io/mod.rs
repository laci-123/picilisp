use crate::{memory::*, util::string_to_list};
use crate::util::list_to_string;
use crate::error_utils::*;
use super::NativeFunctionMetaData;
use std::io::{self, BufRead};
use std::io::prelude::*;



pub const OUTPUT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      output,
    name:          "output",
    kind:          FunctionKind::Lambda,
    parameters:    &["string"],
    documentation: "Print the string `string` to standard output.
Error if `string` is not a valid string."
};

pub fn output(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_arguments(mem, OUTPUT.name, &vec![ParameterType::Any], args)?;
    
    if let Some(msg) = list_to_string(args[0].clone()) {
        println!("{msg}");
    }
    else {
        let error = make_error(mem, "invalid-string", OUTPUT.name, &vec![]);
        return Err(error);
    }

    Ok(mem.symbol_for("ok"))
}


pub const INPUT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      input,
    name:          "input",
    kind:          FunctionKind::Lambda,
    parameters:    &["prompt"],
    documentation: "Print `prompt` to standard output without a newline,
then read a line from standard input."
};

pub fn input(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_arguments(mem, OUTPUT.name, &vec![ParameterType::Any], args)?;
    
    if let Some(prompt) = list_to_string(args[0].clone()) {
        print!("{prompt}");
        io::stdout().flush().unwrap();
    }
    else {
        return Err(make_error(mem, "invalid-string", INPUT.name, &vec![]));
    }

    let stdin = io::stdin();
    let mut line = String::new();
    let status = stdin.lock().read_line(&mut line);

    match status {
        Err(_) => {
            Err(make_error(mem, "input-error", INPUT.name, &vec![]))
        },
        Ok(0) => Err(mem.symbol_for("eof")),
        Ok(_) => Ok(string_to_list(mem, &line)),
    }
}

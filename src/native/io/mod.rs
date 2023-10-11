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
    validate_args!(mem, OUTPUT.name, args, (let msg: TypeLabel::String));
    
    let x = writeln!(mem.stdout.write().expect("RwLock poisoned"), "{msg}");
    match x {
        Ok(_)    => Ok(mem.symbol_for("ok")),
        Err(err) => {
            let vec = vec![("details", string_to_list(mem, &err.kind().to_string()))];
            Err(make_error(mem, "cannot-write-file", OUTPUT.name, &vec))
        },
    }
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
    validate_args!(mem, INPUT.name, args, (let prompt: TypeLabel::String));
    
    print!("{prompt}");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    let status = stdin.lock().read_line(&mut line);

    match status {
        Err(err) => {
            let details = vec![("details", string_to_list(mem, &err.kind().to_string()))];
            Err(make_error(mem, "cannot-read-file", INPUT.name, &details))
        },
        Ok(0) => Err(make_error(mem, "eof", INPUT.name, &vec![])),
        Ok(_) => Ok(string_to_list(mem, &line)),
    }
}


pub const INPUT_FILE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      input_file,
    name:          "input-file",
    kind:          FunctionKind::Lambda,
    parameters:    &["path"],
    documentation: "Read the whole contents of file at `path` into a string"
};

pub fn input_file(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, INPUT_FILE.name, args, (let path: TypeLabel::String));
    
    match std::fs::read_to_string(path) {
        Ok(string) => Ok(string_to_list(mem, &string)),
        Err(err)   => {
            let details = string_to_list(mem, &err.kind().to_string());
            Err(make_error(mem, "cannot-read-file", INPUT_FILE.name, &vec![("details", details)]))
        },
    }
}


pub const OUTPUT_FILE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      output_file,
    name:          "output-file",
    kind:          FunctionKind::Lambda,
    parameters:    &["path", "string"],
    documentation: "Append `string` to the file at `path`."
};

pub fn output_file(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, OUTPUT_FILE.name, args, (let path: TypeLabel::String), (let string: TypeLabel::String));
    
    match std::fs::OpenOptions::new().append(true).open(path) {
        Ok(mut file)   => {
            match writeln!(file, "{string}") {
                Ok(_)    => Ok(mem.symbol_for("ok")),
                Err(err) => {
                    let details = string_to_list(mem, &err.kind().to_string());
                    Err(make_error(mem, "cannot-write-file", OUTPUT_FILE.name, &vec![("details", details)]))
                },
            }
        },
        Err(err)   => {
            let details = string_to_list(mem, &err.kind().to_string());
            Err(make_error(mem, "cannot-write-file", OUTPUT_FILE.name, &vec![("details", details)]))
        },
    }
}

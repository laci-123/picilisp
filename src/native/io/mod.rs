use crate::{memory::*, util::string_to_list};
use crate::util::*;
use crate::error_utils::*;
use super::NativeFunctionMetaData;
use std::io::BufReader;
use std::io::prelude::*;



pub const INPUT_FILE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      input_file,
    name:          "input-file",
    kind:          FunctionKind::Lambda,
    parameters:    &["path"],
    documentation: "Read the whole contents of file at `path` into a string"
};

pub fn input_file(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, INPUT_FILE.name, args, (let input_source: TypeLabel::Any));

    if symbol_eq!(input_source, mem.symbol_for("*stdin*")) {
        let mut line = String::new();
        let mut stdin = BufReader::new(mem.stdin.as_mut());

        loop {
            match stdin.read_line(&mut line) {
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::TimedOut {
                        if let Some(umb) = &mem.umbilical {
                            if let Ok(msg) = umb.from_high_end.try_recv() {
                                match msg.get("command").map(|s| s.as_str()) {
                                    Some("INTERRUPT") => {
                                        return Err(make_error(mem, "interrupted", INPUT_FILE.name, &vec![]));
                                    },
                                    Some("ABORT") => {
                                        return Err(GcRef::nil());
                                    },
                                    _ => {},
                                }
                            }
                        }
                        continue;
                    }
                    else {
                        let details = vec![("details", string_to_list(mem, &err.kind().to_string()))];
                        return Err(make_error(mem, "cannot-read-file", INPUT_FILE.name, &details));
                    }
                },
                Ok(0) => return Err(make_error(mem, "eof", INPUT_FILE.name, &vec![])),
                Ok(_) => return Ok(string_to_list(mem, &line)),
            }
        }
    }
    else {
        let Some(path) = list_to_string(input_source.clone()) else {
            let error_details = vec![("expected", mem.symbol_for("string-type")), ("actual", mem.symbol_for(input_source.get_type().to_string()))];
            return Err(make_error(mem, "wrong-argument-type", INPUT_FILE.name, &error_details));
        };
        match std::fs::read_to_string(path) {
            Ok(string) => Ok(string_to_list(mem, &string)),
            Err(err)   => {
                let details = string_to_list(mem, &err.kind().to_string());
                Err(make_error(mem, "cannot-read-file", INPUT_FILE.name, &vec![("details", details)]))
            },
        }
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
    validate_args!(mem, OUTPUT_FILE.name, args, (let output_source: TypeLabel::Any), (let string: TypeLabel::String));

    if symbol_eq!(output_source, mem.symbol_for("*stdout*")) {
        let status = 
        write!(mem.stdout, "{string}").and_then(|_| {
            mem.stdout.flush()
        });
        match status {
            Ok(_)    => {
                Ok(mem.symbol_for("ok"))
            },
            Err(err) => {
                let vec = vec![("details", string_to_list(mem, &err.kind().to_string()))];
                Err(make_error(mem, "cannot-write-file", OUTPUT_FILE.name, &vec))
            },
        }
    }
    else { 
        let Some(path) = list_to_string(output_source.clone()) else {
            let error_details = vec![("expected", mem.symbol_for("string-type")), ("actual", mem.symbol_for(output_source.get_type().to_string()))];
            return Err(make_error(mem, "wrong-argument-type", OUTPUT_FILE.name, &error_details));
        };
        match std::fs::OpenOptions::new().append(true).open(path) {
            Ok(mut file)   => {
                match write!(file, "{string}") {
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
}

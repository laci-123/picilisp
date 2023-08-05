#![allow(dead_code)]

use crate::memory::*;


enum NativeResult {
    Value(GcRef),
    Signal(GcRef),
    Abort(String),
}


fn call_native_function(mem: &mut Memory, name: &str, arguments: &[GcRef]) -> NativeResult {
    match name {
        "read" => {
            if arguments.len() == 1 {
                NativeResult::Value(read::read(mem, arguments[0].clone()))
            }
            else {
                NativeResult::Signal(mem.symbol_for("wrong-number-of-arguments"))
            }
        },
        "print" => {
            if arguments.len() == 1 {
                NativeResult::Value(print::print(mem, arguments[0].clone()))
            }
            else {
                NativeResult::Signal(mem.symbol_for("wrong-number-of-arguments"))
            }
        },
        _ => NativeResult::Signal(mem.symbol_for("unknown-native-function")),
    }
}



mod print;
mod read;
mod eval;

use crate::memory::*;
use crate::util::list_to_string;



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

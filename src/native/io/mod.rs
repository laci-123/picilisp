use crate::memory::*;
use crate::util::list_to_string;
use crate::native::signal::{make_error, fit_to_number};



pub fn message(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        let error_details = vec![("expected", mem.allocate_number(1)), ("actual", fit_to_number(mem, args.len()))];
        let error = make_error(mem, "wrong-number-of-arguments", "message", &error_details);
        return NativeResult::Signal(error);
    }
    
    if let Some(msg) = list_to_string(args[0].clone()) {
        println!("{msg}");
    }
    else {
        let error = make_error(mem, "invalid-string", "message", &vec![]);
        return NativeResult::Signal(error);
    }

    NativeResult::Value(mem.symbol_for("ok"))
}

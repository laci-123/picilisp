use crate::memory::*;
use crate::util::list_to_string;
use crate::error_utils::*;



pub fn message(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "message", &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
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

use crate::memory::*;
use crate::native::eval::eval;



pub fn define(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    
    let name = args[0].get().as_symbol().get_name();

    let value = 
    match eval(mem, &[args[1].clone()], env) {
        NativeResult::Value(x) => x,
        other                  => return other,
    };

    if mem.is_global_defined(&name) {
        return NativeResult::Value(mem.symbol_for("already-defined"));
    }
    
    mem.define_global(&name, value);

    NativeResult::Value(mem.symbol_for("ok"))
}


pub fn undefine(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    let name = args[0].get().as_symbol().get_name();

    mem.undefine_global(&name);

    NativeResult::Value(mem.symbol_for("ok"))
}
    

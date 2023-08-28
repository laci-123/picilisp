use crate::memory::*;
use crate::native::eval::eval;



pub fn define(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if args[0].is_nil() {
        return NativeResult::Signal(mem.symbol_for("definition-not-symbol"));
    }

    let name =
    if let Some(PrimitiveValue::Symbol(symbol)) = args[0].get() {
        symbol.get_name()
    }
    else {
        return NativeResult::Signal(mem.symbol_for("definition-not-symbol"));
    };

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

    if args[0].is_nil() {
        return NativeResult::Signal(mem.symbol_for("definition-not-symbol"));
    }

    let name =
    if let Some(PrimitiveValue::Symbol(symbol)) = args[0].get() {
        symbol.get_name()
    }
    else {
        return NativeResult::Signal(mem.symbol_for("definition-not-symbol"));
    };

    mem.undefine_global(&name);

    NativeResult::Value(mem.symbol_for("ok"))
}
    

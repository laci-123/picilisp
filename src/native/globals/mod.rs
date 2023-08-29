use crate::memory::*;
use crate::native::eval::eval;
use crate::util::list_to_string;



pub fn define(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 3 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
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

    let documentation =
    if !args[2].is_nil() {
        if let Some(string) = list_to_string(args[2].clone()) {
            string
        }
        else {
            return NativeResult::Signal(mem.symbol_for("documentation-not-string"));
        }
    }
    else {
        return NativeResult::Signal(mem.symbol_for("documentation-not-string"));
    };

    if mem.is_global_defined(&name) {
        return NativeResult::Value(mem.symbol_for("already-defined"));
    }

    if let Some(meta) = args[0].get_metadata() {
        let mut new_meta       = meta.clone();
        new_meta.documentation = documentation;
        let with_meta          = mem.allocate_metadata(value, new_meta);
        mem.define_global(&name, with_meta);
    }
    else {
        mem.define_global(&name, value);
    }

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
    

use crate::{memory::*, util::list_to_vec};



pub fn signal(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    
    NativeResult::Signal(args[0].clone())
}


pub fn trap(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    
    let trap = mem.allocate_trap(args[0].clone(), args[1].clone());

    NativeResult::Value(trap)
}


pub fn get_signal_name(stack_trace: GcRef) -> Option<String> {
    if stack_trace.is_nil() {
        return None;
    }
    
    if let PrimitiveValue::Symbol(symbol) = stack_trace.get() {
        return Some(symbol.get_name());
    }

    if let Some(entries) = list_to_vec(stack_trace) {
        if let Some(last_entry) = entries.last() {
            if let PrimitiveValue::Symbol(symbol) = last_entry.get() {
                return Some(symbol.get_name());
            }
        }
    }

    None
}

use crate::memory::*;



pub fn add(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let PrimitiveValue::Number(x) = args[0].get() {
        if let PrimitiveValue::Number(y) = args[1].get() {
            NativeResult::Value(mem.allocate_number(*x + *y))
        }
        else {
            NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
        }
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}


pub fn multiply(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let PrimitiveValue::Number(x) = args[0].get() {
        if let PrimitiveValue::Number(y) = args[1].get() {
            NativeResult::Value(mem.allocate_number(*x * *y))
        }
        else {
            NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
        }
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}


pub fn divide(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let PrimitiveValue::Number(x) = args[0].get() {
        if let PrimitiveValue::Number(y) = args[1].get() {
            NativeResult::Value(mem.allocate_number(*x / *y))
        }
        else {
            NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
        }
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}

use crate::memory::*;



pub fn add(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let Some(PrimitiveValue::Number(x)) = args[0].get() {
        if let Some(PrimitiveValue::Number(y)) = args[1].get() {
            if let Some(z) = x.checked_add(*y) {
                NativeResult::Value(mem.allocate_number(z))
            }
            else {
                NativeResult::Signal(mem.symbol_for("addition-overflow"))
            }
        }
        else {
            NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
        }
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}


pub fn substract(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let Some(PrimitiveValue::Number(x)) = args[0].get() {
        if let Some(PrimitiveValue::Number(y)) = args[1].get() {
            if let Some(z) = x.checked_sub(*y) {
                NativeResult::Value(mem.allocate_number(z))
            }
            else {
                NativeResult::Signal(mem.symbol_for("substraction-overflow"))
            }
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

    if let Some(PrimitiveValue::Number(x)) = args[0].get() {
        if let Some(PrimitiveValue::Number(y)) = args[1].get() {
            if let Some(z) = x.checked_mul(*y) {
                NativeResult::Value(mem.allocate_number(z))
            }
            else {
                NativeResult::Signal(mem.symbol_for("multiplication-overflow"))
            }
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

    if let Some(PrimitiveValue::Number(x)) = args[0].get() {
        if let Some(PrimitiveValue::Number(y)) = args[1].get() {
            if *y == 0 {
                NativeResult::Signal(mem.symbol_for("divide-by-zero"))
            }
            else {
                NativeResult::Value(mem.allocate_number(*x / *y))
            }
        }
        else {
            NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
        }
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}

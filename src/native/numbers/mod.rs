use crate::memory::*;
use crate::error_utils::*;



pub fn add(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "add", &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
    if nr.is_err() {
        return nr;
    }

    let x = args[0].get().unwrap().as_number();
    let y = args[1].get().unwrap().as_number();
    if let Some(z) = x.checked_add(*y) {
        NativeResult::Value(mem.allocate_number(z))
    }
    else {
        NativeResult::Signal(mem.symbol_for("addition-overflow"))
    }
}


pub fn substract(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "substract", &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
    if nr.is_err() {
        return nr;
    }

    let x = args[0].get().unwrap().as_number();
    let y = args[1].get().unwrap().as_number();
    if let Some(z) = x.checked_sub(*y) {
        NativeResult::Value(mem.allocate_number(z))
    }
    else {
        NativeResult::Signal(mem.symbol_for("substraction-overflow"))
    }
}


pub fn multiply(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "multiply", &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
    if nr.is_err() {
        return nr;
    }

    let x = args[0].get().unwrap().as_number();
    let y = args[1].get().unwrap().as_number();
    if let Some(z) = x.checked_mul(*y) {
        NativeResult::Value(mem.allocate_number(z))
    }
    else {
        NativeResult::Signal(mem.symbol_for("multiplication-overflow"))
    }
}


pub fn divide(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "divide", &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
    if nr.is_err() {
        return nr;
    }

    let x = args[0].get().unwrap().as_number();
    let y = args[1].get().unwrap().as_number();
    if *y == 0 {
        NativeResult::Signal(mem.symbol_for("divide-by-zero"))
    }
    else {
        NativeResult::Value(mem.allocate_number(*x / *y))
    }
}

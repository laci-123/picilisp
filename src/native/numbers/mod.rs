use crate::memory::*;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub const ADD: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      add,
    name:          "add",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return the sum of `x` and `y`.
Error if the additon results in overflow."
};

pub fn add(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, ADD.name, &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
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


pub const SUBSTRACT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      substract,
    name:          "substract",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return the difference of `x` and `y`.
Error if the substraction results in overflow."
};

pub fn substract(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, SUBSTRACT.name, &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
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


pub const MULTIPLY: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      multiply,
    name:          "multiply",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return the product of `x` and `y`.
Error if the multiplication results in overflow."
};

pub fn multiply(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, MULTIPLY.name, &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
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


pub const DIVIDE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      divide,
    name:          "divide",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return the quotient of `x` and `y`.
Error if `y` is 0."
};

pub fn divide(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, DIVIDE.name, &vec![ParameterType::Type(TypeLabel::Number), ParameterType::Type(TypeLabel::Number)], args);
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

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

pub fn add(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, ADD.name, args, (let x: TypeLabel::Number), (let y: TypeLabel::Number));

    if let Some(z) = x.checked_add(*y) {
        Ok(mem.allocate_number(z))
    }
    else {
        Err(make_error(mem, "arithmetic-overflow", ADD.name, &vec![]))
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

pub fn substract(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, SUBSTRACT.name, args, (let x: TypeLabel::Number), (let y: TypeLabel::Number));

    if let Some(z) = x.checked_sub(*y) {
        Ok(mem.allocate_number(z))
    }
    else {
        Err(make_error(mem, "arithmetic-overflow", SUBSTRACT.name, &vec![]))
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

pub fn multiply(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, MULTIPLY.name, args, (let x: TypeLabel::Number), (let y: TypeLabel::Number));

    if let Some(z) = x.checked_mul(*y) {
        Ok(mem.allocate_number(z))
    }
    else {
        Err(make_error(mem, "arithmetic-overflow", MULTIPLY.name, &vec![]))
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

pub fn divide(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, DIVIDE.name, args, (let x: TypeLabel::Number), (let y: TypeLabel::Number));

    if *y == 0 {
        Err(make_error(mem, "divide-by-zero", DIVIDE.name, &vec![]))
    }
    else {
        Ok(mem.allocate_number(*x / *y))
    }
}


pub const LESS: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      less,
    name:          "<",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return t if and only if `x` is less than `y`."
};

pub fn less(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, LESS.name, args, (let x: TypeLabel::Number), (let y: TypeLabel::Number));

    if *x < *y {
        Ok(mem.symbol_for("t"))
    }
    else {
        Ok(GcRef::nil())
    }
}


pub const GREATER: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      greater,
    name:          ">",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return t if and only if `x` is greater than `y`."
};

pub fn greater(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, GREATER.name, args, (let x: TypeLabel::Number), (let y: TypeLabel::Number));

    if *x > *y {
        Ok(mem.symbol_for("t"))
    }
    else {
        Ok(GcRef::nil())
    }
}


#[cfg(test)]
mod tests;

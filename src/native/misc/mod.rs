use crate::memory::*;
use crate::util::{list_to_vec, list_to_string};
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub const GENSYM: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      gensym,
    name:          "gensym",
    kind:          FunctionKind::Lambda,
    parameters:    &[],
    documentation: "GENerate a unique SYMbol.
It is guaranteed that there never has been and never will be a symbol
that is equal to the returned symbol."
};

pub fn gensym(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, GENSYM.name, &vec![], args);
    if nr.is_err() {
        return nr;
    }
    
    NativeResult::Value(mem.unique_symbol())
}


pub const QUOTE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      quote,
    name:          "quote",
    kind:          FunctionKind::SpecialLambda,
    parameters:    &["object"],
    documentation: "Don't do anything with `object`.
Useful if you want to prevent `object` from being evaluated."
};

pub fn quote(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, QUOTE.name, &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    NativeResult::Value(args[0].clone())
}


pub const BRANCH: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      branch,
    name:          "branch",
    kind:          FunctionKind::Lambda,
    parameters:    &["condition", "then", "otherwise"],
    documentation: "Return `otherwise` if `condition` is nil, return `then` if `condition` is anything else."
};

pub fn branch(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, BRANCH.name, &vec![ParameterType::Any, ParameterType::Any, ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let condition = args[0].clone();
    let then      = args[1].clone();
    let otherwise = args[2].clone();

    if !condition.is_nil() {
        NativeResult::Value(then)
    }
    else {
        NativeResult::Value(otherwise)
    }
}


pub const EQUAL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      equal,
    name:          "=",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return`t` if `x` and `y` are equal in type and value, otherwise return nil."
};

pub fn equal(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, EQUAL.name, &vec![ParameterType::Any, ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    NativeResult::Value(if equal_internal(args[0].clone(), args[1].clone()) {mem.symbol_for("t")} else {GcRef::nil()})
}

fn equal_internal(a: GcRef, b: GcRef) -> bool {
    if let (Some(x), Some(y)) = (a.get(), b.get()) {
        match x {
            PrimitiveValue::Number(n1) => {
                if y.is_nil() {
                    false
                }
                else if let PrimitiveValue::Number(n2) = y {
                    *n1 == *n2
                }
                else {
                    false
                }
            },
            PrimitiveValue::Character(c1) => {
                if y.is_nil() {
                    false
                }
                else if let PrimitiveValue::Character(c2) = y {
                    *c1 == *c2
                }
                else {
                    false
                }
            },
            PrimitiveValue::Symbol(s1) => {
                if y.is_nil() {
                    false
                }
                else if let PrimitiveValue::Symbol(s2) = y {
                    *s1 == *s2
                }
                else {
                    false
                }
            },
            PrimitiveValue::Cons(c1) => {
                if y.is_nil() {
                    false
                }
                else if let PrimitiveValue::Cons(c2) = y {
                    if let Some(l1) = list_to_vec(a.clone()) {
                        if let Some(l2) = list_to_vec(b.clone()) {
                            let mut i = 0;
                            let mut j = 0;
                            let mut equal = true;

                            while i < l1.len() && j < l2.len() {
                                // TODO: very deeply nested lists could cause stack overflow
                                if !equal_internal(l1[i].clone(), l2[j].clone()) {
                                    equal = false;
                                    break;
                                }

                                i += 1;
                                j += 1;
                            }

                            equal && i == l1.len() && j == l2.len()
                        }
                        else {
                            // if one of them is a list but the other isn't then they can't be equal
                            false
                        }
                    }
                    else {
                        equal_internal(c1.get_car(), c2.get_car()) && equal_internal(c1.get_cdr(), c2.get_cdr())
                    }
                }
                else {
                    false
                }
            },
            // functions and traps are not equal to anything
            _ => false,
        }
    }
    else {
        a.is_nil() && b.is_nil()
    }
}


pub const ABORT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      abort,
    name:          "abort",
    kind:          FunctionKind::Lambda,
    parameters:    &["message"],
    documentation: "Print `message` to standard output then
immediately abort the execution of the whole program.
If `message` is not a valid string
then immediately abort the execution of the whole program without printing anything."
};

pub fn abort(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, ABORT.name, &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let msg = list_to_string(args[0].clone());

    match msg {
        Some(msg) => NativeResult::Abort(msg),
        None      => NativeResult::Abort("#<invalid-string>".to_string()),
    }
}


#[cfg(test)]
mod tests;

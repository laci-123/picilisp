use crate::memory::*;
use crate::util::list_to_vec;
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

pub fn gensym(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, GENSYM.name, args);
    
    Ok(mem.unique_symbol())
}


pub const EQUAL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      equal,
    name:          "=",
    kind:          FunctionKind::Lambda,
    parameters:    &["x", "y"],
    documentation: "Return`t` if `x` and `y` are equal in type and value, otherwise return nil."
};

pub fn equal(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, EQUAL.name, args, (let x: TypeLabel::Any), (let y: TypeLabel::Any));

    Ok(if equal_internal(x, y) {mem.symbol_for("t")} else {GcRef::nil()})
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
                                // TODO: Very deeply nested lists could cause stack overflow. Use recursion_depth to check.
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


#[cfg(test)]
mod tests;

use crate::memory::*;
use crate::native::eval::eval;
use crate::util::{list_to_vec, list_to_string};



pub fn gensym(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 0 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    
    NativeResult::Value(mem.unique_symbol())
}


pub fn quote(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    NativeResult::Value(args[0].clone())
}

    // can't call it `if`
pub fn branch(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 3 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    let test      = args[0].clone();
    let then      = args[1].clone();
    let otherwise = args[2].clone();

    let evaled_test =
    match eval(mem, &[test], env.clone()) {
        NativeResult::Value(x) => x,
        other                  => return other,
    };

    if !evaled_test.is_nil() {
        eval(mem, &[then], env)
    }
    else {
        eval(mem, &[otherwise], env)
    }
}


pub fn equal(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
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


pub fn abort(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    let msg = list_to_string(args[0].clone());

    match msg {
        Some(msg) => NativeResult::Abort(msg),
        None      => NativeResult::Abort("#<invalid-string>".to_string()),
    }
}


#[cfg(test)]
mod tests;

use crate::memory::*;
use crate::native::eval::eval;
use crate::util::{list_to_vec, list_to_string};


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

fn equal_internal(x: GcRef, y: GcRef) -> bool {
    x.is_nil() && y.is_nil() ||
    match x.get() {
        PrimitiveValue::Number(n1) => {
            if y.is_nil() {
                false
            }
            else if let PrimitiveValue::Number(n2) = y.get() {
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
            else if let PrimitiveValue::Character(c2) = y.get() {
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
            else if let PrimitiveValue::Symbol(s2) = y.get() {
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
            else if let PrimitiveValue::Cons(c2) = y.get() {
                if let Some(l1) = list_to_vec(x.clone()) {
                    if let Some(l2) = list_to_vec(y.clone()) {
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

use crate::memory::*;
use crate::util::list_to_vec;
use crate::native::signal::{make_error, fit_to_number};



fn function(mem: &mut Memory, args: &[GcRef], env: GcRef, kind: FunctionKind) -> NativeResult {
    if args.len() == 2 {
        if let Some(params) = list_to_vec(args[0].clone()) {
            let mut actual_params   = vec![];
            let mut has_rest_params = false;
            let rest_param_symbol   = mem.symbol_for("&");

            let param_count = params.len();
            let mut i = 0;

            for param in params {
                if let Some(PrimitiveValue::Symbol(symbol)) = param.get() {
                    if has_rest_params {
                        actual_params.push(param.clone());
                        break;
                    }

                    if symbol == rest_param_symbol.get().unwrap().as_symbol() {
                        // i == param_count - 2  (rearranged to avoid underflow when param_count == 0)
                        if i + 2 == param_count {
                            //          ---4---
                            //          0 1 2 3
                            // (lambda (x y & z) ...
                            //              ^
                            //              4 - 2
                            has_rest_params = true;
                        }
                        //      i > param_count - 2
                        else if i + 2 > param_count {
                            //          ---4---
                            //          0 1 2 3
                            // (lambda (x y z &) ...
                            //                ^
                            //                3 > 4 - 2
                            let error = make_error(mem, "missing-rest-parameter", "function", &vec![]);
                            return NativeResult::Signal(error);
                        }
                        // i < param_count - 2
                        else {
                            //          ---4---
                            //          0 1 2 3
                            // (lambda (x & y z) ...
                            //            ^
                            //            1 < 4 - 2
                            let error = make_error(mem, "multiple-rest-parameters", "function", &vec![]);
                            return NativeResult::Signal(error);
                        }
                    }
                    else {
                        actual_params.push(param.clone());
                    }
                }
                else {
                    let error_details = vec![("param", param)];
                    let error = make_error(mem, "param-is-not-symbol", "function", &error_details);
                    return NativeResult::Signal(error);
                }

                i += 1;
            }

            let body     = args[1].clone();
            let function = mem.allocate_normal_function(kind, has_rest_params, body, &actual_params, env);
            NativeResult::Value(function)
        }
        else {
            let error = make_error(mem, "bad-param-list", "function", &vec![]);
            NativeResult::Signal(error)
        }
    }
    else {
        let error_details = vec![("expected", mem.allocate_number(2)), ("actual", fit_to_number(mem, args.len()))];
        let error = make_error(mem, "wrong-number-of-arguments", "function", &error_details);
        NativeResult::Signal(error)
    }
}


pub fn lambda        (mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    function(mem, args, env, FunctionKind::Lambda)
}

pub fn special_lambda(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    function(mem, args, env, FunctionKind::SpecialLambda)
}

pub fn macro_macro   (mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    function(mem, args, env, FunctionKind::Macro)
}

pub fn syntax        (mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    function(mem, args, env, FunctionKind::Syntax)
}


#[cfg(test)]
mod tests;

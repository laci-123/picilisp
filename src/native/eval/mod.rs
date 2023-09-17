use crate::memory::*;
use crate::util::*;
use crate::native::read::read;
use crate::native::list::property;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



fn lookup(mem: &mut Memory, key: GcRef, environment: GcRef) -> Option<GcRef> {
    let mut cursor = environment;

    while let Some(c) = cursor.get() {
        let cons = c.as_conscell();
        let key_value = cons.get_car();

        if key_value.get().unwrap().as_conscell().get_car().get().unwrap().as_symbol() == key.get().unwrap().as_symbol() {
            return Some(key_value.get().unwrap().as_conscell().get_cdr());
        }

        cursor = cons.get_cdr();
    }

    mem.get_global(&key.get().unwrap().as_symbol().get_name())
}


fn pair_params_and_args(mem: &mut Memory, nf: &NormalFunction, nf_name: Option<String>, args: &[GcRef], env: GcRef) -> NativeResult {
    let mut new_env =
    if nf.get_kind() == FunctionKind::SpecialLambda {
        env
    }
    else {
        nf.get_env()
    };

    let source = if let Some(name) = nf_name {
        name
    }
    else {
        "#<function>".to_string()
    };

    let mut i = 0;
    for param in nf.non_rest_params() {
        let arg; 
        if let Some(a) = args.get(i) {
            arg = a.clone();
        }
        else {
            let error_details = vec![("expected", fit_to_number(mem, i + 1)), ("actual", fit_to_number(mem, args.len()))];
            let error = make_error(mem, "wrong-number-of-arguments", &source, &error_details);
            return NativeResult::Signal(error);
        };
        let param_arg = mem.allocate_cons(param, arg);
        new_env       = mem.allocate_cons(param_arg, new_env);

        i += 1;
    }

    if let Some(rest_param) = nf.rest_param() {
        let rest_args = vec_to_list(mem, &args[i..]);
        let param_arg = mem.allocate_cons(rest_param, rest_args);
        new_env       = mem.allocate_cons(param_arg, new_env);
    }
    else if i < args.len() {
        let error_details = vec![("expected", fit_to_number(mem, i)), ("actual", fit_to_number(mem, args.len()))];
        let error = make_error(mem, "wrong-number-of-arguments", &source, &error_details);
        return NativeResult::Signal(error);
    }

    NativeResult::Value(new_env)
}


const MAX_STACK_SIZE: usize = 1000;


fn eval_internal(mem: &mut Memory, expression: GcRef, env: GcRef) -> NativeResult {
    let name = expression.get_metadata().map(|md| md.read_name.clone());

    if let Some(mut vec) = list_to_vec(expression.clone()) {
        if let Some(first) = vec.get(0).map(|x| x.clone()) {
            let operator =
            match eval_internal(mem, first, env.clone()) {
                NativeResult::Value(x) => x,
                other => return other,
            };

            if let Some(PrimitiveValue::Function(f)) = operator.get() {
                let eval_args =
                match f.get_kind() {
                    FunctionKind::Lambda                       => true,
                    FunctionKind::SpecialLambda                => false,
                    FunctionKind::Macro | FunctionKind::Syntax => return NativeResult::Signal(mem.symbol_for("eval-found-macro")),
                };

                if eval_args {
                    for i in 1..vec.len() {
                        vec[i] =
                        match eval_internal(mem, vec[i].clone(), env.clone()) {
                            NativeResult::Value(x) => x,
                            other => return other,
                        };
                    }
                }

                match f {
                    Function::NativeFunction(nf) => nf.call(mem, &vec[1..], env.clone()),
                    Function::NormalFunction(nf) => {
                        let new_env =
                        match pair_params_and_args(mem, &nf, name, &vec[1..], env) {
                            NativeResult::Value(x) => x,
                            other => return other,
                        };
                        eval_internal(mem, nf.get_body(), new_env)
                    },
                }
            }
            else {
                let error_details = vec![("symbol", vec[0].clone())];
                let error = make_error(mem, "eval-bad-operator", "eval", &error_details);
                NativeResult::Signal(error)
            }
        }
        else {
            NativeResult::Value(GcRef::nil())
        }
    }
    else {
        match expression.get() {
            Some(PrimitiveValue::Cons(cons)) => {
                let car =
                match eval_internal(mem, cons.get_car(), env.clone()) {
                    NativeResult::Value(x) => x,
                    other => return other,
                };
                let cdr =
                match eval_internal(mem, cons.get_cdr(), env.clone()) {
                    NativeResult::Value(x) => x,
                    other => return other,
                };

                NativeResult::Value(mem.allocate_cons(car, cdr))
            },
            Some(PrimitiveValue::Trap(trap)) => {
                match eval_internal(mem, trap.get_normal_body(), env.clone()) {
                    NativeResult::Signal(signal) => {
                        let key       = mem.symbol_for("*trapped-signal*");
                        let param_arg = mem.allocate_cons(key, signal);
                        let new_env   = mem.allocate_cons(param_arg, env);
                        eval_internal(mem, trap.get_trap_body(), new_env)
                    },
                    other => other,
                }
            },
            Some(PrimitiveValue::Symbol(_)) => {
                if let Some(value) = lookup(mem, expression.clone(), env) {
                    NativeResult::Value(value)
                }
                else {
                    let error_details = vec![("symbol", expression)];
                    let error = make_error(mem, "unbound-symbol", "eval", &error_details);
                    NativeResult::Signal(error)
                }
            },
            _ => {
                NativeResult::Value(expression)
            },
        }
    }
}


fn macroexpand_internal(mem: &mut Memory, expression: GcRef, env: GcRef) -> NativeResult {
    let name = expression.get_metadata().map(|md| md.read_name.clone());

    if let Some(mut vec) = list_to_vec(expression.clone()) {
        if let Some(first) = vec.get(0).map(|x| x.clone()) {
            let operator =
            match macroexpand_internal(mem, first, env.clone()) {
                NativeResult::Value(x) => x,
                other => return other,
            };

            if let Some(PrimitiveValue::Function(f)) = operator.get() {
                if f.get_kind() != FunctionKind::Macro {
                    return NativeResult::Value(expression)
                }

                for i in 1..vec.len() {
                    vec[i] =
                    match macroexpand_internal(mem, vec[i].clone(), env.clone()) {
                        NativeResult::Value(x) => x,
                        other => return other,
                    };
                }

                match f {
                    Function::NativeFunction(nf) => nf.call(mem, &vec[1..], env.clone()),
                    Function::NormalFunction(nf) => {
                        let new_env =
                        match pair_params_and_args(mem, &nf, name, &vec[1..], env) {
                            NativeResult::Value(x) => x,
                            other => return other,
                        };
                        eval_internal(mem, nf.get_body(), new_env)
                    },
                }
            }
            else {
                NativeResult::Value(expression)
            }
        }
        else {
            NativeResult::Value(GcRef::nil())
        }
    }
    else {
        match expression.get() {
            Some(PrimitiveValue::Cons(cons)) => {
                let car =
                match macroexpand_internal(mem, cons.get_car(), env.clone()) {
                    NativeResult::Value(x) => x,
                    other => return other,
                };
                let cdr =
                match macroexpand_internal(mem, cons.get_cdr(), env.clone()) {
                    NativeResult::Value(x) => x,
                    other => return other,
                };

                NativeResult::Value(mem.allocate_cons(car, cdr))
            },
            Some(PrimitiveValue::Symbol(_)) => {
                if let Some(value) = lookup(mem, expression.clone(), env) {
                    if let Some(PrimitiveValue::Function(f)) = value.get() {
                        if f.get_kind() == FunctionKind::Macro {
                            return NativeResult::Value(value);
                        }
                    }
                }
                NativeResult::Value(expression)
            },
            _ => {
                NativeResult::Value(expression)
            },
        }
    }
}


pub const EVAL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      eval,
    name:          "eval",
    kind:          FunctionKind::Lambda,
    parameters:    &["object"],
    documentation: "Expand macros in `object` then evaluate `object`."
};

pub fn eval(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, EVAL.name, &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let expanded =
    match macroexpand_internal(mem, args[0].clone(), env.clone()) {
        NativeResult::Value(e) => e,
        other                  => return other,
    };

    eval_internal(mem, expanded, env)
}


pub const MACROEXPAND: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      macroexpand,
    name:          "macroexpand",
    kind:          FunctionKind::Lambda,
    parameters:    &["object"],
    documentation: "Expand macros in `object`."
};

pub fn macroexpand(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, MACROEXPAND.name, &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    macroexpand_internal(mem, args[0].clone(), env)
}


pub fn eval_external(mem: &mut Memory, tree: GcRef) -> Result<GcRef, String> {
    let empty_env = GcRef::nil();
    
    match eval(mem, &[tree], empty_env.clone()) {
        NativeResult::Value(x)       => Ok(x),
        NativeResult::Signal(signal) => Err(format!("Unhandled signal: {}", list_to_string(crate::native::print::print(mem, &[signal], empty_env).unwrap()).unwrap())),
        NativeResult::Abort(msg)     => Err(msg),
    }
}


pub const LOAD_ALL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      load_all,
    name:          "load-all",
    kind:          FunctionKind::Lambda,
    parameters:    &["string"],
    documentation: "Read, macroexpand and evaluate all expressions in `string` in sequential order.
Error if `string` is not a valid string."
};

pub fn load_all(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, LOAD_ALL.name, &vec![ParameterType::Any, ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");

    let mut input  = args[0].clone();
    let source     = args[1].clone();
    let mut line   = mem.allocate_number(1);
    let mut column = mem.allocate_number(1);

    while !input.is_nil() {
        let output     = match read(mem, &[input.clone(), source.clone(), line.clone(), column.clone()], GcRef::nil()) {
            NativeResult::Value(x) => x,
            other                  => return other,
        };
        let status     = property(mem, "status", output.clone()).unwrap();
        let result     = property(mem, "result", output.clone()).unwrap();
        let rest       = property(mem, "rest",   output.clone()).unwrap();
        let read_error = property(mem, "error",   output.clone()).unwrap();
        line           = property(mem, "line",   output.clone()).unwrap();
        column         = property(mem, "column", output).unwrap();

        if symbol_eq!(status, ok_symbol) {
            let nr = eval(mem, &[result], GcRef::nil());
            if nr.is_err() {
                return nr;
            }
        }
        else if symbol_eq!(status, incomplete_symbol) {
            let error = make_error(mem, "input-incomplete", "load-all", &vec![]);
            return NativeResult::Signal(error);
        }
        else if symbol_eq!(status, error_symbol) {
            let error_details = vec![("details", read_error)];
            let error = make_error(mem, "read-error", "load-all", &error_details);
            return NativeResult::Signal(error);
        }
        else if symbol_eq!(status, invalid_symbol) {
            let error = make_error(mem, "input-invalid-string", "load-all", &vec![]);
            return NativeResult::Signal(error);
        }

        input = rest;
    }

    NativeResult::Value(ok_symbol)
}


#[cfg(test)]
mod tests;

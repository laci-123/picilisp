use crate::memory::*;
use crate::util::*;
use crate::native::read::read;
use crate::native::list::property;
use crate::error_utils::*;
use crate::config;
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


fn pair_params_and_args(mem: &mut Memory, nf: &NormalFunction, nf_name: Option<String>, args: &[GcRef], env: GcRef) -> Result<GcRef, GcRef> {
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
            return Err(error);
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
        return Err(error);
    }

    Ok(new_env)
}


fn eval_internal(mem: &mut Memory, mut expression: GcRef, mut env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", EVAL.name, &vec![]));
    }

    // loop is only used to jump back to the beginning of the function (using `continue`); never runs until the end more than once
    loop { 
        let name = expression.get_metadata().map(|md| md.read_name.clone());

        if let Some(mut list_elems) = list_to_vec(expression.clone()) {
            // `expression` is a list
            
            if let Some(first) = list_elems.get(0).map(|x| x.clone()) {
                // `expression` is a non-empty list
                
                let operator = eval_internal(mem, first, env.clone(), recursion_depth + 1)?;

                if let Some(PrimitiveValue::Function(f)) = operator.get() {
                    // first element of `expression` evaluates to a function
                    
                    let eval_args =
                    match f.get_kind() {
                        FunctionKind::Lambda                       => true,
                        FunctionKind::SpecialLambda                => false,
                        FunctionKind::Macro | FunctionKind::Syntax => {
                            // this should never happen because `macroexpand` is always called at the beginning of `eval`
                            return Err(mem.symbol_for("eval-found-macro"));
                        },
                    };

                    if eval_args {
                        for i in 1..list_elems.len() {
                            list_elems[i] = eval_internal(mem, list_elems[i].clone(), env.clone(), recursion_depth + 1)?;
                        }
                    }

                    match f {
                        Function::NativeFunction(nf) => {
                            if nf.is_the_same_as(eval) {
                                // prevent `eval` from calling itself as regular native function;
                                // instead "reuse" this instance of `eval`
                                validate_arguments(mem, EVAL.name, &vec![ParameterType::Any], &list_elems[1..])?;
                                expression = list_elems[1].clone();
                                continue;
                            }
                            else {
                                return nf.call(mem, &list_elems[1..], env.clone(), recursion_depth + 1)
                            }
                        },
                        Function::NormalFunction(nf) => {
                            // tail-call elimination: jump back to the beginning of this instance of `eval`
                            // instead of calling itself recursively
                            let new_env = pair_params_and_args(mem, &nf, name, &list_elems[1..], env)?;
                            expression = nf.get_body();
                            env = new_env;
                            continue;
                        },
                    }
                }
                else {
                    // first element of `expression` doesn't evaluate to a function
                    let error_details = vec![("symbol", list_elems[0].clone())];
                    let error = make_error(mem, "eval-bad-operator", EVAL.name, &error_details);
                    return Err(error);
                }
            }
            else {
                // `expression` is the empty list
                return Ok(GcRef::nil());
            }
        }
        else {
            // `expression` is not a list
            
            match expression.get() {
                Some(PrimitiveValue::Cons(cons)) => {
                    let car = eval_internal(mem, cons.get_car(), env.clone(), recursion_depth + 1)?;
                    let cdr = eval_internal(mem, cons.get_cdr(), env.clone(), recursion_depth + 1)?;
                    return Ok(mem.allocate_cons(car, cdr));
                },
                Some(PrimitiveValue::Trap(trap)) => {
                    match eval_internal(mem, trap.get_normal_body(), env.clone(), recursion_depth + 1) {
                        Err(signal) => {
                            let key       = mem.symbol_for("*trapped-signal*");
                            let param_arg = mem.allocate_cons(key, signal);
                            let new_env   = mem.allocate_cons(param_arg, env);
                            return eval_internal(mem, trap.get_trap_body(), new_env, recursion_depth + 1);
                        },
                        Ok(x) => return Ok(x),
                    }
                },
                Some(PrimitiveValue::Symbol(_)) => {
                    if let Some(value) = lookup(mem, expression.clone(), env) {
                        return Ok(value);
                    }
                    else {
                        let error_details = vec![("symbol", expression)];
                        let error = make_error(mem, "unbound-symbol", "eval", &error_details);
                        return Err(error);
                    }
                },
                _ => {
                    return Ok(expression);
                },
            }
        }
    }
}


fn macroexpand_internal(mem: &mut Memory, expression: GcRef, env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", MACROEXPAND.name, &vec![]));
    }
    
    let name = expression.get_metadata().map(|md| md.read_name.clone());

    if let Some(mut list_elems) = list_to_vec(expression.clone()) {
        // `expression` is a list
        
        if let Some(first) = list_elems.get(0).map(|x| x.clone()) {
            // `expression` is a non-empty list
            
            let operator = macroexpand_internal(mem, first, env.clone(), recursion_depth + 1)?;

            // expand all elements regardless what the operator is
            for i in 1..list_elems.len() {
                list_elems[i] = macroexpand_internal(mem, list_elems[i].clone(), env.clone(), recursion_depth + 1)?;
            }

            // if the operator is a macro then evaluate it... 
            if let Some(PrimitiveValue::Function(f)) = operator.get() {
                if f.get_kind() == FunctionKind::Macro {
                    match f {
                        Function::NativeFunction(nf) => {
                            return nf.call(mem, &list_elems[1..], env.clone(), recursion_depth + 1);
                        },
                        Function::NormalFunction(nf) => {
                            let new_env = pair_params_and_args(mem, &nf, name, &list_elems[1..], env.clone())?;
                            return eval_internal(mem, nf.get_body(), new_env, recursion_depth + 1);
                        },
                    }
                }
            }

            // ...otherwise return the whole list as-is
            Ok(vec_to_list(mem, &list_elems))
        }
        else {
            // `expression` is the empty list
            Ok(GcRef::nil())
        }
    }
    else {
        // `expression` is not a list
        
        match expression.get() {
            Some(PrimitiveValue::Cons(cons)) => {
                let car = macroexpand_internal(mem, cons.get_car(), env.clone(), recursion_depth + 1)?;
                let cdr = macroexpand_internal(mem, cons.get_cdr(), env.clone(), recursion_depth + 1)?;
                Ok(mem.allocate_cons(car, cdr))
            },
            Some(PrimitiveValue::Symbol(_)) => {
                // only evaluate symbols if their value is a macro
                if let Some(value) = lookup(mem, expression.clone(), env) {
                    if let Some(PrimitiveValue::Function(f)) = value.get() {
                        if f.get_kind() == FunctionKind::Macro {
                            return Ok(value);
                        }
                    }
                }
                Ok(expression)
            },
            _ => {
                Ok(expression)
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

pub fn eval(mem: &mut Memory, args: &[GcRef], env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_arguments(mem, EVAL.name, &vec![ParameterType::Any], args)?;

    let expanded = macroexpand_internal(mem, args[0].clone(), env.clone(), recursion_depth + 1)?;
    eval_internal(mem, expanded, env, recursion_depth + 1)
}


pub const MACROEXPAND: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      macroexpand,
    name:          "macroexpand",
    kind:          FunctionKind::Lambda,
    parameters:    &["object"],
    documentation: "Expand macros in `object`."
};

pub fn macroexpand(mem: &mut Memory, args: &[GcRef], env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_arguments(mem, MACROEXPAND.name, &vec![ParameterType::Any], args)?;

    macroexpand_internal(mem, args[0].clone(), env, recursion_depth + 1)
}


pub fn eval_external(mem: &mut Memory, tree: GcRef) -> Result<GcRef, String> {
    let empty_env = GcRef::nil();
    let recursion_depth = 0;
    
    match eval(mem, &[tree], empty_env.clone(), recursion_depth) {
        Ok(x)       => Ok(x),
        Err(signal) => Err(format!("Unhandled signal: {}", list_to_string(crate::native::print::print(mem, &[signal], empty_env, recursion_depth).ok().unwrap()).unwrap())),
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

pub fn load_all(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_arguments(mem, LOAD_ALL.name, &vec![ParameterType::Any, ParameterType::Any], args)?;

    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");

    let mut input  = args[0].clone();
    let source     = args[1].clone();
    let mut line   = mem.allocate_number(1);
    let mut column = mem.allocate_number(1);

    while !input.is_nil() {
        let output     = read(mem, &[input.clone(), source.clone(), line.clone(), column.clone()], GcRef::nil(), recursion_depth + 1)?;
        let status     = property(mem, "status", output.clone()).unwrap();
        let result     = property(mem, "result", output.clone()).unwrap();
        let rest       = property(mem, "rest",   output.clone()).unwrap();
        let read_error = property(mem, "error",   output.clone()).unwrap();
        line           = property(mem, "line",   output.clone()).unwrap();
        column         = property(mem, "column", output).unwrap();

        if symbol_eq!(status, ok_symbol) {
            let nr = eval(mem, &[result], GcRef::nil(), recursion_depth + 1);
            if nr.is_err() {
                return nr;
            }
        }
        else if symbol_eq!(status, incomplete_symbol) {
            let error = make_error(mem, "input-incomplete", "load-all", &vec![]);
            return Err(error);
        }
        else if symbol_eq!(status, error_symbol) {
            let error_details = vec![("details", read_error)];
            let error = make_error(mem, "read-error", "load-all", &error_details);
            return Err(error);
        }
        else if symbol_eq!(status, invalid_symbol) {
            let error = make_error(mem, "input-invalid-string", "load-all", &vec![]);
            return Err(error);
        }

        input = rest;
    }

    Ok(ok_symbol)
}


#[cfg(test)]
mod tests;

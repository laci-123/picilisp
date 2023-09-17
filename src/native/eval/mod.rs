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


const MAX_RECURSION_DEPTH: usize = 1000;


fn eval_internal(mem: &mut Memory, mut expression: GcRef, mut env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", EVAL.name, &vec![]));
    }
    
    loop {
        let name = expression.get_metadata().map(|md| md.read_name.clone());

        if let Some(mut vec) = list_to_vec(expression.clone()) {
            if let Some(first) = vec.get(0).map(|x| x.clone()) {
                let operator = eval_internal(mem, first, env.clone(), recursion_depth + 1)?;

                if let Some(PrimitiveValue::Function(f)) = operator.get() {
                    let eval_args =
                    match f.get_kind() {
                        FunctionKind::Lambda                       => true,
                        FunctionKind::SpecialLambda                => false,
                        FunctionKind::Macro | FunctionKind::Syntax => {
                            return Err(mem.symbol_for("eval-found-macro"));
                        },
                    };

                    if eval_args {
                        for i in 1..vec.len() {
                            vec[i] = eval_internal(mem, vec[i].clone(), env.clone(), recursion_depth + 1)?;
                        }
                    }

                    match f {
                        Function::NativeFunction(nf) => {
                            if nf.is_the_same_as(eval) {
                                validate_arguments(mem, EVAL.name, &vec![ParameterType::Any], &vec[1..])?;
                                expression = vec[1].clone();
                                continue;
                            }
                            else {
                                return nf.call(mem, &vec[1..], env.clone(), recursion_depth + 1)
                            }
                        },
                        Function::NormalFunction(nf) => {
                            let new_env = pair_params_and_args(mem, &nf, name, &vec[1..], env)?;
                            expression = nf.get_body();
                            env = new_env;
                            continue;
                        },
                    }
                }
                else {
                    let error_details = vec![("symbol", vec[0].clone())];
                    let error = make_error(mem, "eval-bad-operator", "eval", &error_details);
                    return Err(error);
                }
            }
            else {
                return Ok(GcRef::nil());
            }
        }
        else {
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
    if recursion_depth > MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", MACROEXPAND.name, &vec![]));
    }
    
    let name = expression.get_metadata().map(|md| md.read_name.clone());

    if let Some(mut vec) = list_to_vec(expression.clone()) {
        if let Some(first) = vec.get(0).map(|x| x.clone()) {
            let operator = macroexpand_internal(mem, first, env.clone(), recursion_depth + 1)?;

            for i in 1..vec.len() {
                vec[i] = macroexpand_internal(mem, vec[i].clone(), env.clone(), recursion_depth + 1)?;
            }

            if let Some(PrimitiveValue::Function(f)) = operator.get() {
                if f.get_kind() == FunctionKind::Macro {
                    match f {
                        Function::NativeFunction(nf) => nf.call(mem, &vec[1..], env.clone(), recursion_depth + 1),
                        Function::NormalFunction(nf) => {
                            let new_env = pair_params_and_args(mem, &nf, name, &vec[1..], env.clone())?;
                            eval_internal(mem, nf.get_body(), new_env, recursion_depth + 1)
                        },
                    }
                }
                else {
                    Ok(vec_to_list(mem, &vec))
                }
            }
            else {
                Ok(vec_to_list(mem, &vec))
            }
        }
        else {
            Ok(GcRef::nil())
        }
    }
    else {
        match expression.get() {
            Some(PrimitiveValue::Cons(cons)) => {
                let car = macroexpand_internal(mem, cons.get_car(), env.clone(), recursion_depth + 1)?;
                let cdr = macroexpand_internal(mem, cons.get_cdr(), env.clone(), recursion_depth + 1)?;
                Ok(mem.allocate_cons(car, cdr))
            },
            Some(PrimitiveValue::Symbol(_)) => {
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

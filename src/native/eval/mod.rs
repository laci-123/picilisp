use crate::memory::*;
use crate::util::*;
use crate::errors::Error;
use crate::native::read::read;
use crate::native::list::property;
use crate::error_utils::*;
use crate::config;
use super::NativeFunctionMetaData;



fn lookup(mem: &mut Memory, key: GcRef, environment: GcRef, environment_module: &str) -> Result<GcRef, Error> {
    let mut cursor = environment;

    while let Some(c) = cursor.get() {
        let cons = c.as_conscell();
        let key_value = cons.get_car();

        if key_value.get().unwrap().as_conscell().get_car().get().unwrap().as_symbol() == key.get().unwrap().as_symbol() {
            return Ok(key_value.get().unwrap().as_conscell().get_cdr());
        }

        cursor = cons.get_cdr();
    }

    mem.get_global(&key.get().unwrap().as_symbol().get_name(), environment_module)
}


fn pair_params_and_args(mem: &mut Memory, nf: &NormalFunction, nf_name: Option<String>, args: &[GcRef]) -> Result<GcRef, GcRef> {
    let mut new_env = nf.get_env();

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


pub const CALL_NATIVE_FUNCTION: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      call_native_function,
    name:          "call-native-function",
    kind:          FunctionKind::Lambda,
    parameters:    &["function", "arguments", "environment"],
    documentation: "Call the native function `function` with `arguments` and `environment` as its local environment."
};

pub fn call_native_function(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", CALL_NATIVE_FUNCTION.name, &vec![]));
    }
    validate_args!(mem, CALL_NATIVE_FUNCTION.name, args, (let function: TypeLabel::Function), (let arguments: TypeLabel::List), (let environment: TypeLabel::Any));

    if let Function::NativeFunction(nf) = function {
        nf.call(mem, &arguments, environment, recursion_depth + 1)
    }
    else {
        let details = vec![("expected", mem.symbol_for("native-function")), ("actual", mem.symbol_for("normal-function"))];
        Err(make_error(mem, "wrong-argument", CALL_NATIVE_FUNCTION.name, &details))
    }
}


pub const MAKE_TRAP: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      make_trap,
    name:          "make-trap",
    kind:          FunctionKind::Lambda,
    parameters:    &["normal-body", "trap-body"],
    documentation: "Manually construct a trap object from `normal-body` and `trap-body`."
};

pub fn make_trap(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", MAKE_TRAP.name, &vec![]));
    }
    validate_args!(mem, MAKE_TRAP.name, args, (let normal_body: TypeLabel::Any), (let trap_body: TypeLabel::Any));

    Ok(mem.allocate_trap(normal_body, trap_body))
}


pub const MAKE_FUNCTION: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      make_function,
    name:          "make-function",
    kind:          FunctionKind::Lambda,
    parameters:    &["params", "body", "environment", "environment-module", "kind"],
    documentation: "Manually construct a function-object form the given arguments.
`kind` is either `lambda-type` or `macro-type`."
};

pub fn make_function(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", MAKE_FUNCTION.name, &vec![]));
    }
    validate_args!(mem, MAKE_FUNCTION.name, args, (let _params: TypeLabel::List), (let _body: TypeLabel::Any), (let environment: TypeLabel::Any), (let env_module: TypeLabel::Symbol), (let kind: TypeLabel::Symbol));

    let (s, k) =
    match kind.get_name().as_str() {
        "lambda-type"         => ("lambda", FunctionKind::Lambda),
        "macro-type"          => ("macro", FunctionKind::Macro),
        _ => return Err(make_error(mem, "wrong-arg-value", MAKE_FUNCTION.name, &vec![])),
    };

    make_function_internal(mem, &args[0..2], environment, &env_module.get_name(), s, k)
}


fn make_function_internal(mem: &mut Memory, args: &[GcRef], env: GcRef, env_module: &str, source: &str, kind: FunctionKind) -> Result<GcRef, GcRef> {
    validate_args!(mem, source, args, (let params: TypeLabel::List), (let body: TypeLabel::Any));
    
    let mut actual_params   = vec![];
    let mut has_rest_params = false;
    let rest_param_symbol   = mem.symbol_for("&");
    let param_count         = params.len();
    let mut i               = 0;

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
                    let error = make_error(mem, "missing-rest-parameter", source, &vec![]);
                    return Err(error);
                }
                // i < param_count - 2
                else {
                    //          ---4---
                    //          0 1 2 3
                    // (lambda (x & y z) ...
                    //            ^
                    //            1 < 4 - 2
                    let error = make_error(mem, "multiple-rest-parameters", source, &vec![]);
                    return Err(error);
                }
            }
            else {
                actual_params.push(param.clone());
            }
        }
        else {
            let error_details = vec![("param", param)];
            let error = make_error(mem, "param-is-not-symbol", source, &error_details);
            return Err(error);
        }

        i += 1;
    }

    let function = mem.allocate_normal_function(kind, has_rest_params, body, &actual_params, env, env_module);
    Ok(function)
}

fn eval_internal(mem: &mut Memory, mut expression: GcRef, mut env: GcRef, mut env_module: String, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", EVAL.name, &vec![]));
    }

    // loop is only used to jump back to the beginning of the function (using `continue`); never runs until the end more than once
    loop { 
        if let Some(umb) = &mem.umbilical {
            if let Ok(msg) = umb.from_high_end.try_recv() {
                match msg.get("command").map(|s| s.as_str()) {
                    Some("INTERRUPT") => {
                        return Err(make_error(mem, "interrupted", EVAL.name, &vec![]));
                    },
                    Some("ABORT") => {
                        return Err(GcRef::nil());
                    },
                    _ => {},
                }
            }
        }

        let name = expression.get_meta().map(|md| md.read_name.clone());

        if let Some(mut list_elems) = list_to_vec(expression.clone()) {
            // `expression` is a list
            
            if let Some(first) = list_elems.get(0).map(|x| x.clone()) {
                // `expression` is a non-empty list

                if symbol_eq!(list_elems[0], mem.symbol_for("lambda")) {
                    return make_function_internal(mem, &list_elems[1..], env.clone(), &env_module, "lambda", FunctionKind::Lambda);
                }
                else if symbol_eq!(list_elems[0], mem.symbol_for("quote")) {
                    validate_args!(mem, "quote", &list_elems[1..], (let x: TypeLabel::Any));
                    return Ok(x);
                }
                else if symbol_eq!(list_elems[0], mem.symbol_for("if")) {
                    validate_args!(mem, "if", &list_elems[1..], (let condition: TypeLabel::Any), (let then: TypeLabel::Any), (let otherwise: TypeLabel::Any));
                    let evaled_condition = eval_internal(mem, condition, env.clone(), env_module.clone(), recursion_depth + 1)?;
                    if !evaled_condition.is_nil() {
                        // tail-call elimination: jump back to the beginning of this instance of `eval`
                        // instead of calling itself recursively
                        expression = then;
                        continue;
                    }
                    else {
                        // tail-call elimination: jump back to the beginning of this instance of `eval`
                        // instead of calling itself recursively
                        expression = otherwise;
                        continue;
                    }
                }
                else if symbol_eq!(list_elems[0], mem.symbol_for("trap")) {
                    validate_args!(mem, "trap", &list_elems[1..], (let normal_body: TypeLabel::Any), (let trap_body: TypeLabel::Any));
                    return Ok(mem.allocate_trap(normal_body, trap_body));
                }
                else {
                    // first element of `expression` is not a special operator
                    
                    let operator = eval_internal(mem, first, env.clone(), env_module.clone(), recursion_depth + 1)?;

                    if let Some(PrimitiveValue::Function(f)) = operator.get() {
                        // first element of `expression` evaluates to a function

                        // evaluate arguments
                        for i in 1..list_elems.len() {
                            list_elems[i] = eval_internal(mem, list_elems[i].clone(), env.clone(), env_module.clone(), recursion_depth + 1)?;
                        }

                        match f {
                            Function::NativeFunction(nf) => {
                                if nf.is_the_same_as(eval) {
                                    // prevent `eval` from calling itself as regular native function;
                                    // instead "reuse" this instance of `eval`
                                    validate_args!(mem, EVAL.name, &list_elems[1..], (let x: TypeLabel::Any));
                                    expression = macroexpand_completely(mem, x, env.clone(), &env_module, recursion_depth + 1)?;
                                    continue;
                                }
                                else {
                                    return nf.call(mem, &list_elems[1..], env.clone(), recursion_depth + 1)
                                }
                            },
                            Function::NormalFunction(nf) => {
                                // tail-call elimination: jump back to the beginning of this instance of `eval`
                                // instead of calling itself recursively
                                let new_env = pair_params_and_args(mem, &nf, name, &list_elems[1..])?;
                                expression = nf.get_body();
                                env = new_env;
                                env_module = nf.get_env_module();
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
                    let car = eval_internal(mem, cons.get_car(), env.clone(), env_module.clone(), recursion_depth + 1)?;
                    let cdr = eval_internal(mem, cons.get_cdr(), env.clone(), env_module, recursion_depth + 1)?;
                    return Ok(mem.allocate_cons(car, cdr));
                },
                Some(PrimitiveValue::Trap(trap)) => {
                    match eval_internal(mem, trap.get_normal_body(), env.clone(), env_module.clone(), recursion_depth + 1) {
                        Err(signal) => {
                            if signal.is_nil() {
                                return Err(signal);
                            }
                            else {
                                let key       = mem.symbol_for("*trapped-signal*");
                                let param_arg = mem.allocate_cons(key, signal);
                                let new_env   = mem.allocate_cons(param_arg, env);
                                return eval_internal(mem, trap.get_trap_body(), new_env, env_module, recursion_depth + 1);
                            }
                        },
                        Ok(x) => return Ok(x),
                    }
                },
                Some(PrimitiveValue::Symbol(_)) => {
                    match lookup(mem, expression.clone(), env, &env_module) {
                        Ok(value) => return Ok(value),
                        Err(Error::AmbiguousName(modules)) => {
                            let conflicting_modules = modules.iter().map(|m| mem.symbol_for(m)).collect::<Vec<GcRef>>();
                            let error_details = vec![("symbol", expression), ("conflicting-modules", vec_to_list(mem, &conflicting_modules))];
                            let error = make_error(mem, "ambiguous-name", EVAL.name, &error_details);
                            return Err(error);
                        },
                        Err(Error::GlobalNonExistentOrPrivate) => {
                            let error_details = vec![("symbol", expression.clone())];
                            let error = make_error(mem, "unbound-symbol", EVAL.name, &error_details);
                            return Err(error);
                            // panic!("{} | {}", crate::native::print::print_to_rust_string(expression.clone(), 0).unwrap(), mem.get_current_module());
                        },
                        _ => unreachable!(),
                    }
                },
                _ => {
                    return Ok(expression);
                },
            }
        }
    }
}


fn macroexpand_internal(mem: &mut Memory, expression: GcRef, env: GcRef, env_module: &str, recursion_depth: usize, changed: &mut bool) -> Result<GcRef, GcRef> {
    if recursion_depth > config::MAX_RECURSION_DEPTH {
        return Err(make_error(mem, "stackoverflow", MACROEXPAND.name, &vec![]));
    }
    
    let name = expression.get_meta().map(|md| md.read_name.clone());

    if let Some(mut list_elems) = list_to_vec(expression.clone()) {
        // `expression` is a list
        
        if let Some(first) = list_elems.get(0).map(|x| x.clone()) {
            // `expression` is a non-empty list
            
            if symbol_eq!(list_elems[0], mem.symbol_for("macro")) {
                return make_function_internal(mem, &list_elems[1..], env.clone(), env_module, "macro", FunctionKind::Macro);
            }
            else if symbol_eq!(list_elems[0], mem.symbol_for("quote")) {
                return Ok(expression);
            }
            else {
                // first element of `expression` is not a special operator

                let operator = macroexpand_internal(mem, first, env.clone(), env_module, recursion_depth + 1, changed)?;

                // expand all elements regardless what the operator is
                for i in 1..list_elems.len() {
                    list_elems[i] = macroexpand_internal(mem, list_elems[i].clone(), env.clone(), env_module, recursion_depth + 1, changed)?;
                }

                // if the operator is a macro then evaluate it... 
                if let Some(PrimitiveValue::Function(f)) = operator.get() {
                    if f.get_kind() == FunctionKind::Macro {
                        *changed = true;
                        match f {
                            Function::NativeFunction(nf) => {
                                return nf.call(mem, &list_elems[1..], env.clone(), recursion_depth + 1);
                            },
                            Function::NormalFunction(nf) => {
                                let new_env = pair_params_and_args(mem, &nf, name, &list_elems[1..])?;
                                return eval_internal(mem, nf.get_body(), new_env, nf.get_env_module(), recursion_depth + 1);
                            },
                        }
                    }
                }

                // ...otherwise return the whole list as-is
                Ok(vec_to_list(mem, &list_elems))
            }
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
                let car = macroexpand_internal(mem, cons.get_car(), env.clone(), env_module, recursion_depth + 1, changed)?;
                let cdr = macroexpand_internal(mem, cons.get_cdr(), env.clone(), env_module, recursion_depth + 1, changed)?;
                Ok(mem.allocate_cons(car, cdr))
            },
            Some(PrimitiveValue::Symbol(_)) => {
                match lookup(mem, expression.clone(), env, &mem.get_current_module()) {
                    Ok(value) => {
                        if let Some(PrimitiveValue::Function(f)) = value.get() {
                            if f.get_kind() == FunctionKind::Macro {
                                *changed = true;
                                return Ok(value);
                            }
                        }
                    },
                    Err(Error::AmbiguousName(modules)) => {
                        let conflicting_modules = modules.iter().map(|m| mem.symbol_for(m)).collect::<Vec<GcRef>>();
                        let error_details = vec![("symbol", expression), ("conflicting-modules", vec_to_list(mem, &conflicting_modules))];
                        let error = make_error(mem, "ambiguous-name", MACROEXPAND.name, &error_details);
                        return Err(error);
                    },
                    Err(Error::GlobalNonExistentOrPrivate) => {
                        // do nothing
                    },
                    _ => unreachable!(),
                }
                Ok(expression)
            },
            _ => {
                Ok(expression)
            },
        }
    }
}


fn macroexpand_completely(mem: &mut Memory, expression: GcRef, env: GcRef, env_module: &str, recursion_depth: usize) -> Result<GcRef, GcRef> {
    let mut expanded = expression.clone();
    loop {
        let mut changed  = false;
        expanded = macroexpand_internal(mem, expanded, env.clone(), env_module, recursion_depth + 1, &mut changed)?;
        if !changed {
            break;
        }
    }

    Ok(expanded)
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
    validate_args!(mem, EVAL.name, args, (let x: TypeLabel::Any));

    let env_module = mem.get_current_module();

    let expanded = macroexpand_completely(mem, x, env.clone(), &env_module, recursion_depth + 1)?;
    eval_internal(mem, expanded, env, env_module, recursion_depth + 1)
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
    validate_args!(mem, MACROEXPAND.name, args, (let x: TypeLabel::Any));

    let env_module = mem.get_current_module();
    macroexpand_completely(mem, x, env.clone(), &env_module, recursion_depth + 1)
}


pub fn eval_external(mem: &mut Memory, tree: GcRef) -> Result<GcRef, String> {
    let empty_env = GcRef::nil();
    let recursion_depth = 0;

    let result =
    match eval(mem, &[tree], empty_env.clone(), recursion_depth) {
        Ok(x)       => Ok(x),
        Err(signal) => {
            if signal.is_nil() {
                Err(format!("Evaluation aborted."))
            }
            else {
                Err(list_to_string(crate::native::print::print(mem, &[signal], empty_env, recursion_depth).ok().unwrap()).unwrap())
            }
        },
    };

    if let Some(umb) = &mut mem.umbilical {
        umb.paused  = false;
        umb.in_step = false;
    }

    result
}


pub const LOAD_ALL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      load_all,
    name:          "load-all",
    kind:          FunctionKind::Lambda,
    parameters:    &["string", "source"],
    documentation: "Read, macroexpand and evaluate all expressions in `string` in sequential order.
Error if `string` is not a valid string."
};

pub fn load_all(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, LOAD_ALL.name, args, (let _input: TypeLabel::String), (let source: TypeLabel::Any));

    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");
    let mut line          = mem.allocate_number(1);
    let mut column        = mem.allocate_number(1);
    let mut cursor        = args[0].clone();

    let old_module = mem.get_current_module();
    if let Some(s) = list_to_string(source.clone()) {
        mem.define_module(&s);
    }

    while !cursor.is_nil() {
        let output     = read(mem, &[cursor.clone(), source.clone(), line.clone(), column.clone()], GcRef::nil(), recursion_depth + 1)?;
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
            let error = make_error(mem, "input-incomplete", LOAD_ALL.name, &vec![]);
            return Err(error);
        }
        else if symbol_eq!(status, error_symbol) {
            let error_details = vec![("details", read_error)];
            let error = make_error(mem, "read-error", LOAD_ALL.name, &error_details);
            return Err(error);
        }
        else if symbol_eq!(status, invalid_symbol) {
            let error = make_error(mem, "input-invalid-string", LOAD_ALL.name, &vec![]);
            return Err(error);
        }

        cursor = rest;
    }

    mem.set_current_module(&old_module).unwrap();

    Ok(ok_symbol)
}


#[cfg(test)]
mod tests;

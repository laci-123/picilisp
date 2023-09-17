use crate::memory::*;
use crate::util::list_to_vec;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



fn function(mem: &mut Memory, args: &[GcRef], env: GcRef, source: &str, kind: FunctionKind) -> Result<GcRef, GcRef> {
    validate_arguments(mem, source, &vec![ParameterType::Any, ParameterType::Any], args)?;
    
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

        let body     = args[1].clone();
        let function = mem.allocate_normal_function(kind, has_rest_params, body, &actual_params, env);
        Ok(function)
    }
    else {
        let error = make_error(mem, "bad-param-list", source, &vec![]);
        Err(error)
    }
}


pub const LAMBDA: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      lambda,
    name:          "lambda",
    kind:          FunctionKind::SpecialLambda,
    parameters:    &["parameters", "body"],
    documentation: "Create a lambda function with `parameters` and `body`.
Lambda functions are evaluated at runtime,
and their arguments are evaluated in left-to-right order before the function itself is evaluated.",
};


pub fn lambda(mem: &mut Memory, args: &[GcRef], env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    function(mem, args, env, LAMBDA.name, FunctionKind::Lambda)
}


pub const SPECIAL_LAMBDA: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      special_lambda,
    name:          "special-lambda",
    kind:          FunctionKind::SpecialLambda,
    parameters:    &["parameters", "body"],
    documentation: "Create a special-lambda function with `parameters` and `body`.
Special-lambda functions are evaluated at runtime,
but their arguments are not evaluated.
They also differ from normal lambda functions in that
they do not capture the environment they are declared in,
instead they have access to the environment they are called in."
};

pub fn special_lambda(mem: &mut Memory, args: &[GcRef], env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    function(mem, args, env, SPECIAL_LAMBDA.name, FunctionKind::SpecialLambda)
}


pub const MACRO: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      macro_macro,
    name:          "macro",
    kind:          FunctionKind::Macro,
    parameters:    &["parameters", "body"],
    documentation: "Create a macro function with `parameters` and `body`.
Macros are expanded before runtime,
and their arguments are not evaluated.",
};

pub fn macro_macro(mem: &mut Memory, args: &[GcRef], env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    function(mem, args, env, MACRO.name, FunctionKind::Macro)
}


pub const SYNTAX: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      syntax,
    name:          "syntax",
    kind:          FunctionKind::Macro,
    parameters:    &["parameters", "body"],
    documentation: "Create a syntax-macro function with `parameters` and `body`.
Syntax-macros are evaluated during reading, even before regular macros.
Their only argument is the source string the reader is currently processing,
and they should return a lisp-object and the rest of the source string.",
};

pub fn syntax(mem: &mut Memory, args: &[GcRef], env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    function(mem, args, env, SYNTAX.name, FunctionKind::Syntax)
}


#[cfg(test)]
mod tests;

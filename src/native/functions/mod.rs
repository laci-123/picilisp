use crate::memory::*;
use crate::util::list_to_vec;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



fn function(mem: &mut Memory, args: &[GcRef], env: GcRef, source: &str, kind: FunctionKind) -> Result<GcRef, GcRef> {
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

    let function = mem.allocate_normal_function(kind, has_rest_params, body, &actual_params, env);
    Ok(function)
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


#[cfg(test)]
mod tests;

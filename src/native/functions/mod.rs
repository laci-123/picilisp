use crate::memory::*;
use crate::util::list_to_vec;


fn function(mem: &mut Memory, args: &[GcRef], env: GcRef, kind: FunctionKind) -> NativeResult {
    if args.len() == 2 {
        if let Some(params) = list_to_vec(args[0].clone()) {
            let body     = args[1].clone();
            let function = mem.allocate_normal_function(kind, body, params, env);
            NativeResult::Value(function)
        }
        else {
            NativeResult::Signal(mem.symbol_for("bad-param-list"))
        }
    }
    else {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        NativeResult::Signal(signal)
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

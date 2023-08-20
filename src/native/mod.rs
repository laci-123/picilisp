#![allow(dead_code)]

use crate::memory::*;



pub fn load_native_functions(mem: &mut Memory) {
    load_native_function(mem, "lambda",   functions::lambda, FunctionKind::SpecialLambda);
    load_native_function(mem, "cons",     list::cons,        FunctionKind::Lambda);
    load_native_function(mem, "car",      list::car,         FunctionKind::Lambda);
    load_native_function(mem, "cdr",      list::cdr,         FunctionKind::Lambda);
    load_native_function(mem, "list",     list::list,        FunctionKind::Lambda);
    load_native_function(mem, "quote",    misc::quote,       FunctionKind::SpecialLambda);
    load_native_function(mem, "if",       misc::branch,      FunctionKind::SpecialLambda);
    load_native_function(mem, "=",        misc::equal,       FunctionKind::Lambda);
    load_native_function(mem, "abort",    misc::abort,       FunctionKind::Lambda);
    load_native_function(mem, "signal",   signal::signal,    FunctionKind::Lambda);
    load_native_function(mem, "trap",     signal::trap,      FunctionKind::SpecialLambda);
    load_native_function(mem, "read",     read::read,        FunctionKind::Lambda);
    load_native_function(mem, "eval",     eval::eval,        FunctionKind::Lambda);
    load_native_function(mem, "print",    print::print,      FunctionKind::Lambda);
    load_native_function(mem, "add",      numbers::add,      FunctionKind::Lambda);
    load_native_function(mem, "multiply", numbers::multiply, FunctionKind::Lambda);
    load_native_function(mem, "divide",   numbers::divide,   FunctionKind::Lambda);
    load_native_function(mem, "define",   globals::define,   FunctionKind::SpecialLambda);
    load_native_function(mem, "undefine", globals::undefine, FunctionKind::SpecialLambda);
    load_native_function(mem, "repl",     repl::repl,        FunctionKind::Lambda);
}

fn load_native_function(mem: &mut Memory, name: &str, function: fn(&mut Memory, &[GcRef], GcRef) -> NativeResult, kind: FunctionKind) {
    let empty_env = GcRef::nil();

    let nf = mem.allocate_native_function(kind, function, empty_env);
    mem.define_global(name, nf);
}



pub mod print;
pub mod read;
pub mod eval;
pub mod functions;
pub mod repl;
pub mod list;
pub mod signal;
pub mod numbers;
pub mod globals;
pub mod misc;

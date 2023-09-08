use crate::memory::*;



pub fn load_native_functions(mem: &mut Memory) {
    load_native_function(mem, "lambda",         functions::lambda,         FunctionKind::SpecialLambda);
    load_native_function(mem, "macro",          functions::macro_macro,    FunctionKind::Macro);
    load_native_function(mem, "special-lambda", functions::special_lambda, FunctionKind::SpecialLambda);
    load_native_function(mem, "syntax",         functions::syntax,         FunctionKind::SpecialLambda);
    load_native_function(mem, "cons",           list::cons,                FunctionKind::Lambda);
    load_native_function(mem, "car",            list::car,                 FunctionKind::Lambda);
    load_native_function(mem, "cdr",            list::cdr,                 FunctionKind::Lambda);
    load_native_function(mem, "list",           list::list,                FunctionKind::Lambda);
    load_native_function2(mem, list::UNREST);
    load_native_function(mem, "get-property",   list::get_property,        FunctionKind::Lambda);
    load_native_function(mem, "gensym",         misc::gensym,              FunctionKind::SpecialLambda);
    load_native_function(mem, "quote",          misc::quote,               FunctionKind::SpecialLambda);
    load_native_function(mem, "if",             misc::branch,              FunctionKind::SpecialLambda);
    load_native_function(mem, "=",              misc::equal,               FunctionKind::Lambda);
    load_native_function(mem, "abort",          misc::abort,               FunctionKind::Lambda);
    load_native_function(mem, "signal",         signal::signal,            FunctionKind::Lambda);
    load_native_function(mem, "trap",           signal::trap,              FunctionKind::SpecialLambda);
    load_native_function2(mem, read::READ);
    load_native_function(mem, "macroexpand",    eval::macroexpand,         FunctionKind::Lambda);
    load_native_function(mem, "eval",           eval::eval,                FunctionKind::Lambda);
    load_native_function(mem, "load-all",       eval::load_all,            FunctionKind::Lambda);
    load_native_function2(mem, print::PRINT);
    load_native_function(mem, "add",            numbers::add,              FunctionKind::Lambda);
    load_native_function(mem, "substract",      numbers::substract,        FunctionKind::Lambda);
    load_native_function(mem, "multiply",       numbers::multiply,         FunctionKind::Lambda);
    load_native_function(mem, "divide",         numbers::divide,           FunctionKind::Lambda);
    load_native_function(mem, "define",         globals::define,           FunctionKind::SpecialLambda);
    load_native_function(mem, "undefine",       globals::undefine,         FunctionKind::SpecialLambda);
    load_native_function(mem, "type-of",        reflection::type_of,       FunctionKind::Lambda);
    load_native_function(mem, "get-metadata",   reflection::get_metadata,  FunctionKind::Lambda);
    load_native_function(mem, "message",        io::message,               FunctionKind::Lambda);
    load_native_function(mem, "repl",           repl::repl,                FunctionKind::Lambda);
}

fn load_native_function(mem: &mut Memory, name: &str, function: fn(&mut Memory, &[GcRef], GcRef) -> NativeResult, kind: FunctionKind) {
    let empty_env = GcRef::nil();

    let nf = mem.allocate_native_function(kind, function, empty_env);
    mem.define_global(name, nf);
}


fn load_native_function2(mem: &mut Memory, nfmd: NativeFunctionMetaData) {
    let empty_env = GcRef::nil();

    let nf = mem.allocate_native_function(nfmd.kind, nfmd.function, empty_env);
    let meta = Metadata{ read_name: nfmd.name.to_string(), location: Location::Native, documentation: nfmd.documentation.to_string(), parameters: nfmd.parameters.iter().map(|s| s.to_string()).collect() };
    let nf_with_meta = mem.allocate_metadata(nf, meta);
    mem.define_global(nfmd.name, nf_with_meta);
}


pub struct NativeFunctionMetaData {
    function: fn (&mut Memory, &[GcRef], GcRef) -> NativeResult,
    name: &'static str,
    kind: FunctionKind,
    documentation: &'static str,
    parameters: &'static [&'static str],
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
pub mod io;
pub mod reflection;
pub mod misc;

use crate::memory::*;


pub fn load_native_functions(mem: &mut Memory) {
    load_native_function(mem, functions::LAMBDA);
    load_native_function(mem, functions::SPECIAL_LAMBDA);
    load_native_function(mem, functions::MACRO);
    load_native_function(mem, functions::SYNTAX);
    load_native_function(mem, list::CONS);
    load_native_function(mem, list::CAR);
    load_native_function(mem, list::CDR);
    load_native_function(mem, list::LIST);
    load_native_function(mem, list::GET_PROPERTY);
    load_native_function(mem, list::UNREST);
    load_native_function(mem, signal::SIGNAL);
    load_native_function(mem, signal::TRAP);
    load_native_function(mem, read::READ);
    load_native_function(mem, eval::MACROEXPAND);
    load_native_function(mem, eval::EVAL);
    load_native_function(mem, eval::LOAD_ALL);
    load_native_function(mem, print::PRINT);
    load_native_function(mem, numbers::ADD);
    load_native_function(mem, numbers::SUBSTRACT);
    load_native_function(mem, numbers::MULTIPLY);
    load_native_function(mem, numbers::DIVIDE);
    load_native_function(mem, globals::DEFINE);
    load_native_function(mem, globals::UNDEFINE);
    load_native_function(mem, reflection::TYPE_OF);
    load_native_function(mem, reflection::GET_METADATA);
    load_native_function(mem, io::INPUT);
    load_native_function(mem, io::OUTPUT);
    load_native_function(mem, misc::GENSYM);
    load_native_function(mem, misc::QUOTE);
    load_native_function(mem, misc::BRANCH);
    load_native_function(mem, misc::EQUAL);
}


fn load_native_function(mem: &mut Memory, nfmd: NativeFunctionMetaData) {
    let empty_env = GcRef::nil();

    let nf = mem.allocate_native_function(nfmd.kind, nfmd.function, empty_env);
    let meta = Metadata{ read_name: nfmd.name.to_string(), location: Location::Native, documentation: nfmd.documentation.to_string(), parameters: nfmd.parameters.iter().map(|s| s.to_string()).collect() };
    let nf_with_meta = mem.allocate_metadata(nf, meta);
    mem.define_global(nfmd.name, nf_with_meta);
}


pub struct NativeFunctionMetaData {
    function: fn (&mut Memory, &[GcRef], GcRef, usize) -> Result<GcRef, GcRef>,
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

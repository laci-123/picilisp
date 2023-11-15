use crate::memory::*;
use crate::debug::*;
use crate::metadata::*;


pub fn load_native_functions(mem: &mut Memory) {
    let old_module = mem.get_current_module();
    mem.define_module("native");

    load_native_function(mem, list::CONS);
    load_native_function(mem, list::CAR);
    load_native_function(mem, list::CDR);
    load_native_function(mem, list::LIST);
    load_native_function(mem, list::GET_PROPERTY);
    load_native_function(mem, list::UNREST);
    load_native_function(mem, signal::ABORT);
    load_native_function(mem, signal::SIGNAL);
    load_native_function(mem, read::READ);
    load_native_function(mem, eval::MAKE_TRAP);
    load_native_function(mem, eval::MAKE_FUNCTION);
    load_native_function(mem, eval::CALL_NATIVE_FUNCTION);
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
    load_native_function(mem, globals::WHEREIS);
    load_native_function(mem, globals::EXPORT);
    load_native_function(mem, globals::GET_CURRENT_MODULE);
    load_native_function(mem, reflection::DESTRUCTURE_TRAP);
    load_native_function(mem, reflection::GET_PARAMETERS);
    load_native_function(mem, reflection::GET_BODY);
    load_native_function(mem, reflection::GET_ENVIRONMENT);
    load_native_function(mem, reflection::TYPE_OF);
    load_native_function(mem, reflection::GET_METADATA);
    load_native_function(mem, debug::SEND);
    load_native_function(mem, debug::RECEIVE);
    load_native_function(mem, io::INPUT);
    load_native_function(mem, io::OUTPUT);
    load_native_function(mem, io::INPUT_FILE);
    load_native_function(mem, io::OUTPUT_FILE);
    load_native_function(mem, misc::GENSYM);
    load_native_function(mem, misc::EQUAL);

    mem.set_current_module(&old_module).unwrap();
}


fn load_native_function(mem: &mut Memory, nfmd: NativeFunctionMetaData) {
    let empty_env = GcRef::nil();
    let md = Metadata {
        read_name:     nfmd.name.to_string(),
        location:      Location::Native,
        documentation: nfmd.documentation.to_string(),
    };
    let nf = mem.allocate_native_function(nfmd.kind, nfmd.parameters.iter().map(|s| s.to_string()).collect(), nfmd.function, empty_env).with_metadata(md);
    mem.define_global(nfmd.name, nf);

    if let Some(umb) = &mem.umbilical {
        let mut dm = DebugMessage::new();
        dm.insert("kind".to_string(), GLOBAL_DEFINED.to_string());
        dm.insert("name".to_string(), nfmd.name.to_string());
        dm.insert("module".to_string(), mem.get_current_module());
        dm.insert("type".to_string(), TypeLabel::Function.to_string().to_string());
        dm.insert("value".to_string(), match nfmd.kind {
            FunctionKind::Macro  => "#<macro>",
            FunctionKind::Lambda => "#<lambda>",
        }.to_string());
        umb.to_high_end.send(dm).expect("supervisor thread disappeared");
    }
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
pub mod list;
pub mod signal;
pub mod numbers;
pub mod globals;
pub mod io;
pub mod reflection;
pub mod debug;
pub mod misc;

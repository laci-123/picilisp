use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use crate::native::eval::eval_external;



fn load(mem: &mut Memory, string: &str, module: &str) -> Result<(), String> {
    let prelude     = string_to_proper_list(mem, string);
    let source_name = string_to_proper_list(mem, module);
    let vec         = vec![mem.symbol_for("load-all"), prelude, source_name];
    let expression  = vec_to_list(mem, &vec);
    eval_external(mem, expression)?;

    Ok(())
}


pub fn load_prelude(mem: &mut Memory) -> Result<(), String> {
    let prelude_str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/prelude.lisp"));  
    load(mem, prelude_str, "*prelude*")
}

pub fn load_debugger(mem: &mut Memory) -> Result<(), String> {
    let prelude_str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/debugger.lisp"));  
    load(mem, prelude_str, "*debugger*")
}

pub fn load_repl(mem: &mut Memory) -> Result<(), String> {
    let prelude_str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/repl.lisp"));  
    load(mem, prelude_str, "*repl*")
}


pub mod gui;
pub mod terminal;

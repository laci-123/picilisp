use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use crate::native::eval::eval_external;



pub fn load_prelude(mem: &mut Memory) -> Result<(), String> {
    // (load-all "prelude contents..." "prelude")
    let prelude_str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/prelude.lisp"));  
    let prelude     = string_to_proper_list(mem, prelude_str);
    let source_name = string_to_proper_list(mem, "prelude");
    let vec         = vec![mem.symbol_for("load-all"), prelude, source_name];
    let expression  = vec_to_list(mem, &vec);
    eval_external(mem, expression)?;

    Ok(())
}


pub mod gui;
pub mod terminal;

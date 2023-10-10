use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use crate::native::eval::eval_external;



pub fn load_prelude(mem: &mut Memory) -> Result<(), String> {
    // (load-all "prelude contents..." (quote prelude))
    let prelude_str = include_str!("../prelude.lisp");  
    let prelude     = string_to_proper_list(mem, prelude_str);
    let source_name = vec![mem.symbol_for("quote"), mem.symbol_for("prelude")];
    let vec         = vec![mem.symbol_for("load-all"), prelude, vec_to_list(mem, &source_name)];
    let expression  = vec_to_list(mem, &vec);
    eval_external(mem, expression)?;

    Ok(())
}


pub mod gui;
pub mod terminal;

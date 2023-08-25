use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use crate::native::eval::eval_external;
use crate::native::load_native_functions;



fn main() {
    println!("Hello, world!");

    let mut mem = Memory::new();

    load_native_functions(&mut mem);

    let input_str = concat!(include_str!("prelude.lisp"),         // load prelude
                            "\n(repl)");                          // then start REPL (\n needed to end any line-comments)
    let input     = string_to_proper_list(&mut mem, input_str);

    let vec = vec![mem.symbol_for("load-all"), input];
    let expression = vec_to_list(&mut mem, &vec);
    
    match eval_external(&mut mem, expression) {
        Ok(_)    => println!("Bye!"),
        Err(msg) => println!("ABORTED:\n{}", msg),
    }
}



mod memory;
mod util;
mod native;

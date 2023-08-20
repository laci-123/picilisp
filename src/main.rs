use crate::memory::*;
use crate::util::vec_to_list;
use crate::native::eval::eval_external;

fn main() {
    println!("Hello, world!");

    let mut mem = Memory::new();

    let repl = mem.allocate_native_function(FunctionKind::Lambda, crate::native::repl::repl);
    mem.define_global("repl", repl);

    let vec = vec![mem.symbol_for("repl")];
    let expression = vec_to_list(&mut mem, vec);

    match eval_external(&mut mem, expression) {
        Ok(_)    => println!("Bye!"),
        Err(msg) => println!("ABORTED:\n{}", msg),
    }
}

mod memory;
mod util;
mod native;

use crate::memory::*;
use crate::util::{vec_to_list, string_to_list, list_to_string};
use crate::native::eval::{eval_external, load_all};
use crate::native::print::print;
use crate::native::load_native_functions;

fn main() {
    println!("Hello, world!");

    let mut mem = Memory::new();

    load_native_functions(&mut mem);

    let prelude_str = include_str!("prelude.lisp");
    let prelude     = string_to_list(&mut mem, prelude_str);

    match load_all(&mut mem, &[prelude], GcRef::nil()) {
        NativeResult::Value(_)   => println!("Loaded prelude."),
        NativeResult::Signal(sig)  => println!("Unhandled signal: {}", list_to_string(print(&mut mem, &[sig], GcRef::nil()).unwrap()).unwrap()),
        NativeResult::Abort(msg) => {
            println!("ABORTED:\n{msg}");
            return;
        },
    }

    let vec = vec![mem.symbol_for("repl")];
    let expression = vec_to_list(&mut mem, &vec);

    match eval_external(&mut mem, expression) {
        Ok(_)    => println!("Bye!"),
        Err(msg) => println!("ABORTED:\n{}", msg),
    }
}

mod memory;
mod util;
mod native;

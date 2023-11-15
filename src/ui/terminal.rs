use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list};
use crate::native::eval::eval_external;
use crate::native::load_native_functions;

pub fn run() -> Result<(), String> {
    let mut mem = Memory::new();

    load_native_functions(&mut mem);
    println!("Loaded native functions.");

    super::load_prelude(&mut mem)?;
    println!("Loaded prelude.");
    super::load_repl(&mut mem)?;
    println!("Loaded repl.");

    // (repl ">>> " nil)
    let vec        = vec![mem.symbol_for("repl"), string_to_proper_list(&mut mem, ">>> "), GcRef::nil()];
    let expression = vec_to_list(&mut mem, &vec);
    eval_external(&mut mem, expression)?;

    println!("Bye!");

    Ok(())
}

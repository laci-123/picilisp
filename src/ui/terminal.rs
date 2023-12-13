use crate::memory::*;
use crate::util::{vec_to_list, string_to_proper_list, list_to_string};
use crate::native::eval::eval_external;
use crate::native::load_native_functions;



pub fn interactive() -> Result<(), String> {
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


pub fn run_command(command: &str) -> Result<String, String> {
    let mut mem = Memory::new();

    load_native_functions(&mut mem);
    super::load_prelude(&mut mem)?;
    super::load_repl(&mut mem)?;

    // (read-eval-print "command" nil)
    let vec        = vec![mem.symbol_for("read-eval-print"), string_to_proper_list(&mut mem, command), GcRef::nil()];
    let expression = vec_to_list(&mut mem, &vec);
    eval_external(&mut mem, expression).map(|x| list_to_string(x).expect("result of read-eval-print is not a string"))
}

pub fn run_file(path: &str) -> Result<(), String> {
    let mut mem = Memory::new();
    load_native_functions(&mut mem);
    super::load_prelude(&mut mem)?;

    // (load "input...")
    let vec        = vec![mem.symbol_for("load"), string_to_proper_list(&mut mem, path)];
    let expression = vec_to_list(&mut mem, &vec);
    let _result    = eval_external(&mut mem, expression)?;

    Ok(())
}

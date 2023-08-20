use crate::memory::*;
use crate::util::vec_to_list;
use crate::native::eval::eval_external;

fn main() {
    println!("Hello, world!");

    let mut mem = Memory::new();

    let empty_env = GcRef::nil();

    let repl = mem.allocate_native_function(FunctionKind::Lambda, crate::native::repl::repl, empty_env.clone());
    mem.define_global("repl", repl);

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, crate::native::functions::lambda, empty_env.clone());
    mem.define_global("lambda", lambda);

    let cons = mem.allocate_native_function(FunctionKind::Lambda, crate::native::list::cons, empty_env.clone());
    mem.define_global("cons", cons);

    let car = mem.allocate_native_function(FunctionKind::Lambda, crate::native::list::car, empty_env.clone());
    mem.define_global("car", car);

    let cdr = mem.allocate_native_function(FunctionKind::Lambda, crate::native::list::cdr, empty_env.clone());
    mem.define_global("cdr", cdr);

    let list = mem.allocate_native_function(FunctionKind::Lambda, crate::native::list::list, empty_env.clone());
    mem.define_global("list", list);

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

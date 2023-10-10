// use crate::memory::*;
// use crate::util::{vec_to_list, string_to_proper_list};
// use crate::native::eval::eval_external;
// use crate::native::load_native_functions;
use eframe::{App, Frame, egui, NativeOptions, run_native};


struct Window {
    program_text: String,
}

impl Window {
    pub fn new() -> Self {
        Self{ program_text: String::new() }
    }
}

impl App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Picilisp");
            ui.label("Hello, World!");
        });
    }
}



fn main() -> Result<(), String> {
    let window = Box::new(Window::new());
    run_native("Picilisp", NativeOptions::default(), Box::new(|_| window)).expect("could not open window");
    // println!("PiciLisp");

    // let mut mem = Memory::new();

    // load_native_functions(&mut mem);

    // println!("Loaded native functions.");

    // // (load-all "prelude contents..." (quote prelude))
    // let prelude_str = include_str!("prelude.lisp");  
    // let prelude     = string_to_proper_list(&mut mem, prelude_str);
    // let source_name = vec![mem.symbol_for("quote"), mem.symbol_for("prelude")];
    // let vec         = vec![mem.symbol_for("load-all"), prelude, vec_to_list(&mut mem, &source_name)];
    // let expression  = vec_to_list(&mut mem, &vec);
    // eval_external(&mut mem, expression)?;

    // println!("Loaded prelude.");

    // // (repl ">>> " nil)
    // let vec        = vec![mem.symbol_for("repl"), string_to_proper_list(&mut mem, ">>> "), GcRef::nil()];
    // let expression = vec_to_list(&mut mem, &vec);
    // eval_external(&mut mem, expression)?;

    // println!("Bye!");

    Ok(())
}



mod metadata;
mod memory;
mod util;
mod native;
mod error_utils;
mod parser;
mod config;

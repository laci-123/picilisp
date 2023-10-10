use memory::*;
use util::{vec_to_list, string_to_proper_list};
use native::eval::eval_external;
use native::load_native_functions;
use eframe::{App, Frame, egui, epaint, NativeOptions, run_native};
use util::list_to_string;


struct Window {
    mem: Memory, 
    program_text: String,
    result_text: String,
    signal_text: Option<String>,
}

impl Window {
    pub fn new() -> Self {
        let mut this = Self{ mem: Memory::new(), program_text: String::new(), result_text: String::new(), signal_text: None };

        load_native_functions(&mut this.mem);

        // (load-all "prelude contents..." (quote prelude))
        let prelude_str = include_str!("prelude.lisp");  
        let prelude     = string_to_proper_list(&mut this.mem, prelude_str);
        let source_name = vec![this.mem.symbol_for("quote"), this.mem.symbol_for("prelude")];
        let vec         = vec![this.mem.symbol_for("load-all"), prelude, vec_to_list(&mut this.mem, &source_name)];
        let expression  = vec_to_list(&mut this.mem, &vec);
        if let Err(err) = eval_external(&mut this.mem, expression) {
            this.signal_text = Some(err);
        }

        this
    }

    pub fn eval(&mut self) {
        self.signal_text = None;
        
        // (read-eval-print "input string")
        let vec        = vec![self.mem.symbol_for("read-eval-print"), string_to_proper_list(&mut self.mem, &self.program_text)];
        let expression = vec_to_list(&mut self.mem, &vec);
        match eval_external(&mut self.mem, expression) {
            Ok(x) => {
                self.result_text = list_to_string(x).unwrap()
            },
            Err(x) => self.signal_text = Some(x),
        }
    }
}

impl App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            ui.spacing_mut().text_edit_width = ui.available_width();
            ui.heading("Picilisp");

            ui.text_edit_multiline(&mut self.program_text);
            ui.add_space(10.0);
            if ui.button("Evaluate").clicked() {
                self.eval();
            }
            ui.add_space(10.0);
            ui.text_edit_multiline(&mut self.result_text);
            if let Some(x) = &self.signal_text {
                // passing a mutable reference to an immutable str to TextEdit::multiline
                // makes it selectable/copyable but not editable
                ui.add(egui::TextEdit::multiline(&mut x.as_str()).text_color(epaint::Color32::RED));
            }
        });
    }
}



fn main() -> Result<(), String> {
    let window = Box::new(Window::new());
    run_native("Picilisp", NativeOptions::default(), Box::new(|_| window)).expect("could not open window");

    Ok(())
}



mod metadata;
mod memory;
mod util;
mod native;
mod error_utils;
mod parser;
mod config;

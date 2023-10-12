use crate::config::GUI_OUTPUT_BUFFER_SIZE;
use crate::memory::*;
use crate::debug::*;
use crate::io::OutputBuffer;
use crate::util::*;
use crate::native::eval::eval_external;
use crate::native::load_native_functions;
use crate::config;
use eframe::{App, Frame, egui, epaint, NativeOptions, run_native};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;



struct Window {
    to_worker: mpsc::Sender<String>,
    from_worker: mpsc::Receiver<Result<String, String>>,
    umbilical_tx: mpsc::Sender<DebugCommand>,
    program_text: String,
    result_text: String,
    signal_text: String,
    output: Arc<RwLock<OutputBuffer>>,
    working: bool,
}

impl Window {
    fn new() -> Self {
        let (to_worker_tx,   to_worker_rx)   = mpsc::channel::<String>();
        let (from_worker_tx, from_worker_rx) = mpsc::channel::<Result<String, String>>();
        let (umbilical_tx, umbilical_rx)     = mpsc::channel::<DebugCommand>();
        let output = Arc::new(RwLock::new(OutputBuffer::new(GUI_OUTPUT_BUFFER_SIZE)));
        let output_clone = Arc::clone(&output);

        thread::spawn(move || {
            let mut mem = Memory::new();
            mem.set_stdout(output_clone);
            mem.attach_umbilical(Umbilical::new(umbilical_rx));

            load_native_functions(&mut mem);

            match super::load_prelude(&mut mem) {
                Ok(_) => {
                    from_worker_tx.send(Ok("\"loaded prelude\"".to_string())).expect("main thread disappeared");
                },
                Err(err) => {
                    from_worker_tx.send(Err(err)).expect("main thread disappeared");
                },
            }

            for input in to_worker_rx.iter() {
                // (read-eval-print "input string...")
                let vec        = vec![mem.symbol_for("read-eval-print"), string_to_proper_list(&mut mem, &input)];
                let expression = vec_to_list(&mut mem, &vec);
                match eval_external(&mut mem, expression) {
                    Ok(x)    => {
                        let output = list_to_string(x).expect("read-eval-print returned something that cannot be converted to a string");
                        from_worker_tx.send(Ok(output)).expect("no receiver for worker-thread messages");
                    },
                    Err(err) => {
                        from_worker_tx.send(Err(err)).expect("main thread disappeared");
                    },
                }
            }
        });

        Self {
            program_text: String::new(),
            result_text: String::new(),
            signal_text: String::new(),
            to_worker: to_worker_tx,
            from_worker: from_worker_rx,
            output,
            umbilical_tx,
            working: true,
        }
    }

    fn update(&mut self) {
        match self.from_worker.try_recv() {
            Ok(Ok(x))    => {
                self.result_text = x;
                self.signal_text.clear();
                self.working = false;
            },
            Ok(Err(err)) => {
                self.signal_text = err;
                self.result_text.clear();
                self.working = false;
            },
            Err(_)       => {},
        }
    }

    fn eval(&mut self) {
        self.signal_text.clear();
        self.to_worker.send(self.program_text.clone()).expect("worker thread dissappeared");
        self.working = true;
    }
}

impl App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            ui.spacing_mut().text_edit_width = ui.available_width();
            ui.heading(config::APPLICATION_NAME);
            ui.add_space(10.0);

            self.update();

            ui.add(egui::TextEdit::multiline(&mut self.program_text).font(egui::FontId::monospace(12.0)));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Evaluate").clicked() {
                    self.eval();
                }
                if ui.button("Force stop").clicked() {
                    self.umbilical_tx.send(DebugCommand::Abort).expect("worker thread dissappeared");
                }
                if self.working {
                    ui.label("working...");
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
            });
            ui.add_space(10.0);

            egui::Frame::none().fill(ui.visuals().extreme_bg_color).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.result_text.as_str()).font(egui::FontId::monospace(12.0)));
            });
            if self.signal_text.len() > 0 {
                // passing a mutable reference to an immutable str to TextEdit::multiline
                // makes it selectable/copyable but not editable
                ui.add(egui::TextEdit::multiline(&mut self.signal_text.as_str()).desired_rows(2).text_color(epaint::Color32::RED));
            }
            ui.add_space(10.0);

            ui.label("Output");
            egui::scroll_area::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                egui::Frame::none().fill(ui.visuals().extreme_bg_color).show(ui, |ui| {
                    let outputbuffer = self.output.read().expect("RwLock poisoned");
                    let mut output = outputbuffer.to_string().expect("invalid unicode in stdout");
                    if outputbuffer.is_truncated() {
                        output.insert_str(0, "...");
                    }
                    ui.text_edit_multiline(&mut output.as_str());
                });
            });
            if ui.button("Clear").clicked() {
                self.output.write().expect("RwLock poisoned").clear();
            }
        });
    }
}


pub fn run() -> Result<(), String> {
    let window = Box::new(Window::new());
    run_native(config::APPLICATION_NAME, NativeOptions::default(), Box::new(|_| window)).map_err(|err| err.to_string())
}

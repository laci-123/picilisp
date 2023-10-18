use crate::config;
use crate::memory::*;
use crate::debug::*;
use crate::io::OutputBuffer;
use crate::util::*;
use crate::native::eval::eval_external;
use crate::native::load_native_functions;
use eframe::{App, Frame, egui, epaint, NativeOptions, run_native};
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;



#[derive(PartialEq, Eq)]
enum WorkerState {
    Ready,
    Working,
    Paused,
}


struct GlobalConstant {
    value: Result<String, String>,
    value_type: String,
}


struct Window {
    to_worker: mpsc::Sender<String>,
    from_worker: mpsc::Receiver<Result<String, String>>,
    umbilical: UmbilicalHighEnd,
    program_text: String,
    result_text: String,
    signal_text: String,
    cursor: usize,
    globals: BTreeMap<String, GlobalConstant>,
    output: Arc<RwLock<OutputBuffer>>,
    free_memory_sapmles: VecDeque<(usize, usize)>,
    used_memory_sapmles: VecDeque<(usize, usize)>,
    used_cells: usize,
    free_cells: usize,
    stack: Vec<Result<String, String>>,
    worker_state: WorkerState,
}

impl Window {
    fn new() -> Self {
        let (to_worker_tx,   to_worker_rx)   = mpsc::channel::<String>();
        let (from_worker_tx, from_worker_rx) = mpsc::channel::<Result<String, String>>();
        let (umbilical_high_end, umbilical_low_end) = make_umbilical();
        let output = Arc::new(RwLock::new(OutputBuffer::new(config::GUI_OUTPUT_BUFFER_SIZE)));
        let output_clone = Arc::clone(&output);

        thread::Builder::new().stack_size(config::CALL_STACK_SIZE).spawn(move || {
            let mut mem = Memory::new();
            mem.set_stdout(output_clone);
            mem.attach_umbilical(umbilical_low_end);

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
        }).expect("could not start worker thread");

        Self {
            program_text: String::new(),
            result_text: String::new(),
            signal_text: String::new(),
            cursor: 0,
            globals: BTreeMap::new(),
            to_worker: to_worker_tx,
            from_worker: from_worker_rx,
            output,
            free_memory_sapmles: VecDeque::with_capacity(100),
            used_memory_sapmles: VecDeque::with_capacity(100),
            used_cells: 0,
            free_cells: 0,
            stack: Vec::new(),
            umbilical: umbilical_high_end,
            worker_state: WorkerState::Working,
        }
    }

    fn update(&mut self) {
        match self.from_worker.try_recv() {
            Ok(Ok(x))    => {
                self.result_text = x;
                self.signal_text.clear();
                self.worker_state = WorkerState::Ready;
            },
            Ok(Err(err)) => {
                self.signal_text = format!("ERROR:\n\n{err}");
                self.result_text.clear();
                self.worker_state = WorkerState::Ready;
            },
            Err(_)       => {},
        }

        match self.umbilical.from_low_end.try_recv() {
            Ok(DiagnosticData::GlobalDefined { name, value, value_type }) => {
                self.globals.insert(name, GlobalConstant {
                    value,
                    value_type: value_type.to_string().to_string(),
                });
            },
            Ok(DiagnosticData::GlobalUndefined { name }) => {
                self.globals.remove(&name);
            },
            Ok(DiagnosticData::Memory { free_cells, used_cells, serial_number }) => {
                if self.free_memory_sapmles.len() > 100 {
                    self.free_memory_sapmles.pop_front();
                }
                self.free_memory_sapmles.push_back((serial_number, free_cells));

                if self.used_memory_sapmles.len() > 100 {
                    self.used_memory_sapmles.pop_front();
                }
                self.used_memory_sapmles.push_back((serial_number, used_cells));

                self.free_cells = free_cells;
                self.used_cells = used_cells;
            },
            Ok(DiagnosticData::CurrentStackFrame { content }) => {
                self.stack.push(content);
            },
            Ok(DiagnosticData::PopStackFrame) => {
                self.stack.pop();
            }
            Err(_) => {},
        }
    }

    fn eval(&mut self) {
        self.signal_text.clear();
        self.to_worker.send(self.program_text.clone()).expect("worker thread dissappeared");
        self.worker_state = WorkerState::Working;
    }
}

impl App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.update();

        egui::SidePanel::left("Left panel").default_width(300.0).show(ctx, |ui| {
            ui.heading("Global constants");
            egui::scroll_area::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (name, value) in self.globals.iter() {
                    ui.collapsing(name, |ui| {
                        let value_label =
                        match &value.value {
                            Ok(x)    => ui.add(egui::Label::new(x).sense(egui::Sense::click())).on_hover_text("click to show metadata"),
                            Err(err) => ui.label(egui::RichText::new(format!("ERROR while converting to string: {err}")).color(epaint::Color32::RED)),
                        };
                        if value_label.clicked() {
                            self.to_worker.send(format!("(describe {name})")).expect("worker thread dissappeared");
                            self.worker_state = WorkerState::Working;
                        }
                        ui.label(&value.value_type);
                    });
                }
            });
            ui.add_space(20.0);

            ui.heading("Memory useage");
            let uc = self.used_cells;
            let fc = self.free_cells;
            let tc = uc + fc;
            let upt = uc as f32 / tc as f32;
            ui.label(format!("Used cells:  {} ({} kb)", uc, uc * CELL_SIZE_BYTES / 1024));
            ui.label(format!("Free cells:  {} ({} kb)", fc, fc * CELL_SIZE_BYTES / 1024));
            ui.label(format!("Total cells: {} ({} kb)", tc, tc * CELL_SIZE_BYTES / 1024));
            ui.label(format!("{:.1}% used", upt * 100.0));

            let free_points = egui_plot::PlotPoints::new(self.free_memory_sapmles.range(..).map(|(x, y)| [*x as f64, *y as f64]).collect());
            let free_line = egui_plot::Line::new(free_points).color(epaint::Color32::GREEN).name("free cells");
            let used_points = egui_plot::PlotPoints::new(self.used_memory_sapmles.range(..).map(|(x, y)| [*x as f64, *y as f64]).collect());
            let used_line = egui_plot::Line::new(used_points).color(epaint::Color32::RED).name("used cells");
            let legend = egui_plot::Legend{ position: egui_plot::Corner::LeftTop, ..Default::default() };
            egui_plot::Plot::new("Memory usage").x_axis_label("samples").y_axis_label("number of cells").allow_scroll(false).legend(legend).show(ui, |plot_ui| {
                plot_ui.line(free_line);
                plot_ui.line(used_line);
                let mut xy = vec![];
                let mut prev_y = 0;
                for (x, y) in self.used_memory_sapmles.range(..) {
                    if *y < prev_y {
                        xy.push([*x as f64, prev_y as f64]);
                    }
                    prev_y = *y;
                }
                let points = egui_plot::Points::new(xy).color(epaint::Color32::BLUE).shape(egui_plot::MarkerShape::Down).radius(5.0).name("GC collect");
                plot_ui.points(points);
            });
        });

        egui::SidePanel::right("Right panel").show(ctx, |ui| {
            ui.heading("Call stack");
            for frame in self.stack.iter() {
                match frame {
                    Ok(x) => {
                        ui.label(x);
                    },
                    Err(err) => {
                        ui.label(egui::RichText::new(err).color(epaint::Color32::RED));
                    },
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            ui.spacing_mut().text_edit_width = ui.available_width();
            ui.heading(config::APPLICATION_NAME);
            ui.add_space(10.0);

            let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                let layout_job = highlight_parens_layout(self.cursor.saturating_sub(1), string);
                ui.fonts(|f| f.layout_job(layout_job))
            };
            let program_textedit = egui::TextEdit::multiline(&mut self.program_text).font(egui::FontId::monospace(12.0)).layouter(&mut layouter).show(ui);
            if let Some(cr) = program_textedit.cursor_range {
                self.cursor = cr.primary.ccursor.index;
            }
            if ui.input(|i| i.key_pressed(egui::Key::F5)) {
                self.eval();
            }
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                match self.worker_state {
                    WorkerState::Ready => {
                        if ui.button("Evaluate").clicked() {
                            self.eval();
                        }
                        ui.add_enabled(false, egui::Button::new("Stop"));
                        ui.add_enabled(false, egui::Button::new("Force Stop"));
                        ui.add_enabled(false, egui::Button::new("Pause"));
                    },
                    WorkerState::Working => {
                        ui.add_enabled(false, egui::Button::new("Evaluate"));
                        if ui.button("Stop").clicked() {
                            self.umbilical.to_low_end.send(DebugCommand::InterruptSignal).expect("worker thread dissappeared");
                        }
                        if ui.button("Force stop").clicked() {
                            self.umbilical.to_low_end.send(DebugCommand::Abort).expect("worker thread dissappeared");
                        }
                        if ui.button("Pause").clicked() {
                            self.worker_state = WorkerState::Paused;
                            self.umbilical.to_low_end.send(DebugCommand::Pause).expect("worker thread dissappeared");
                        }
                        ui.label("working...");
                        ctx.request_repaint();
                    },
                    WorkerState::Paused => {
                        ui.add_enabled(false, egui::Button::new("Evaluate"));
                        if ui.button("Stop").clicked() {
                            self.umbilical.to_low_end.send(DebugCommand::InterruptSignal).expect("worker thread dissappeared");
                        }
                        if ui.button("Force stop").clicked() {
                            self.umbilical.to_low_end.send(DebugCommand::Abort).expect("worker thread dissappeared");
                        }
                        if ui.button("Resume").clicked() {
                            self.worker_state = WorkerState::Working;
                            self.umbilical.to_low_end.send(DebugCommand::Resume).expect("worker thread dissappeared");
                        }
                        ui.label("PAUSED");
                    },
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


struct StringWithCursor<'a> {
    string: &'a str,
    cursor: usize,
}


#[derive(Debug)]
enum HighlightedParens {
    None,
    Ok(usize, usize),
    UnbalancedOpen(usize),
    UnbalancedClose(usize),
}


fn highlight_parens_layout(cursor: usize, text: &str) -> egui::text::LayoutJob {
    let mut layout_job: egui::text::LayoutJob = Default::default();

    match highlight_parens(StringWithCursor { string: text, cursor }) {
        HighlightedParens::None       => layout_job.append(text, 0.0, Default::default()),
        HighlightedParens::Ok(op, cp) => {
            layout_job.append(&text.chars().take(op).collect::<String>(),                       0.0, Default::default());
            layout_job.append(&format!("{}", text.chars().nth(op).unwrap()),                    0.0, egui::text::TextFormat{ color: epaint::Color32::GREEN, ..Default::default() });
            layout_job.append(&text.chars().skip(op + 1).take(cp - op - 1).collect::<String>(), 0.0, Default::default());
            layout_job.append(&format!("{}", text.chars().nth(cp).unwrap()),                    0.0, egui::text::TextFormat{ color: epaint::Color32::GREEN, ..Default::default() });
            layout_job.append(&text.chars().skip(cp + 1).collect::<String>(),                   0.0, Default::default());
        },
        HighlightedParens::UnbalancedOpen(x) | HighlightedParens::UnbalancedClose(x) => {
            layout_job.append(&text.chars().take(x).collect::<String>(),     0.0, Default::default());
            layout_job.append(&format!("{}", text.chars().nth(x).unwrap()),  0.0, egui::text::TextFormat{ color: epaint::Color32::RED, ..Default::default() });
            layout_job.append(&text.chars().skip(x + 1).collect::<String>(), 0.0, Default::default());
        },
    }

    layout_job
}


fn highlight_parens(sc: StringWithCursor) -> HighlightedParens {
    if sc.string.len() == 0 {
        return HighlightedParens::None;
    }
    
    let chars = sc.string.chars().collect::<Vec<char>>();

    let (step, this_paren, matching_paren) =
    match chars.get(sc.cursor) {
        Some('(') =>  (1, '(', ')'),
        Some(')') => (-1, ')', '('),
        _   => return HighlightedParens::None,
    };

    let mut level = 0;
    let mut i = sc.cursor;
    loop {
        let ch = if let Some(c) = chars.get(i) {*c} else {break};

        if ch == this_paren {
            level += 1;
        }
        else if ch == matching_paren {
            level -= 1;
        }

        if level == 0 {
            return HighlightedParens::Ok(sc.cursor.min(i), sc.cursor.max(i));
        }

        i = if let Some(i_plus_step) = i.checked_add_signed(step) {i_plus_step} else {break};
    }


    if step > 0 {
        HighlightedParens::UnbalancedOpen(sc.cursor)
    }
    else {
        HighlightedParens::UnbalancedClose(sc.cursor)
    }
}



#[cfg(test)]
mod tests;

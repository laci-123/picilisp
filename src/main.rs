fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let _program_name = args.next();
    let run_gui =
    if let Some(arg) = args.next() {
        arg.trim().to_lowercase() == "gui"
    }
    else {
        false
    };

    if run_gui {
        ui::gui::run()
    }
    else {
        ui::terminal::run()
    }
}



mod metadata;
mod memory;
mod util;
mod native;
mod error_utils;
mod config;
mod ui;
mod io;
mod debug;

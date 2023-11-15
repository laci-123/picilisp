fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();

    match args.get(1).map(|s| s.as_ref()) {
        None            => ui::terminal::interactive(),
        Some("command") => {
            let command = args.get(2).ok_or("missing command")?;
            ui::terminal::run_command(command)
        },
        Some("gui")     => ui::gui::run(),
        Some(other)     => Err(format!("Unknown command: '{other}'")),
    }
}


mod errors;
mod metadata;
mod memory;
mod util;
mod native;
mod error_utils;
mod config;
mod ui;
mod io;
mod debug;

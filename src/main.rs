fn main() -> Result<(), String> {
    let mut args = std::env::args();

    let _program_name = args.next();

    match args.next().as_deref() {
        None => ui::terminal::interactive(),
        Some("--expression") => {
            let command = args.next().ok_or_else(|| "Missing expression. Use --help flag for help.")?;
            let result = ui::terminal::run_command(&command)?;
            println!("{result}");
            Ok(())
        },
        Some("--load") => {
            let filename = args.next().ok_or_else(|| "Missing filename. Use --help flag for help.")?;
            ui::terminal::run_file(&filename)
        },
        Some("--gui")  => ui::gui::run(),
        Some("--help") => {
            println!("{}", usage());
            Ok(())
        },
        Some(other)    => Err(format!("Unknown command: {other}. Use --help flag for help.")),
    }
}

fn usage() -> String {
    let name = crate::config::APPLICATION_NAME;
    format!("Usage:

{name}                         start interactive REPL
{name} --load <filename>       load the {name}-module defined in <filename>
{name} --expression <expr>     evaluate <expr>, print its result to standard output, then exit
{name} --gui                   start the graphical debugger
{name} --help                  print this help message")
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

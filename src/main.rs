fn main() -> Result<(), String> {
    gui::run()
}



mod metadata;
mod memory;
mod util;
mod native;
mod error_utils;
mod parser;
mod config;
mod gui;

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod assets;
mod lyrics;
mod theme;

fn main() {
    app::run();
}

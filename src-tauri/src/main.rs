// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let config_path = std::env::args().nth(1).map(std::path::PathBuf::from);
    raspkiosk_lib::run_with_config_path(config_path);
}

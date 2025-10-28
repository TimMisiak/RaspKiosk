use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{Manager, Url};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Deserialize)]
struct KioskConfig {
    #[serde(default = "default_start_url")]
    start_url: String,
}

impl Default for KioskConfig {
    fn default() -> Self {
        Self {
            start_url: default_start_url(),
        }
    }
}

fn default_start_url() -> String {
    "https://example.com".to_string()
}

fn config_path(cli_path: Option<&Path>) -> PathBuf {
    if let Some(path) = cli_path {
        return path.to_path_buf();
    }

    let cwd_path = PathBuf::from("kioskconfig.yaml");
    if cwd_path.exists() {
        return cwd_path;
    }

    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        exe_path.push("kioskconfig.yaml");
        if exe_path.exists() {
            return exe_path;
        }
    }

    cwd_path
}

fn load_config(cli_path: Option<&Path>) -> KioskConfig {
    let path = config_path(cli_path);
    match fs::read_to_string(&path) {
        Ok(contents) => match serde_yaml::from_str(&contents) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "Failed to parse kiosk configuration at {}: {err}. Falling back to defaults.",
                    path.display()
                );
                KioskConfig::default()
            }
        },
        Err(err) => {
            eprintln!(
                "Failed to read kiosk configuration at {}: {err}. Falling back to defaults.",
                path.display()
            );
            KioskConfig::default()
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_with_config_path(None);
}

pub fn run_with_config_path(config_path: Option<PathBuf>) {
    let config = load_config(config_path.as_deref());
    let start_url = Url::parse(&config.start_url).unwrap_or_else(|err| {
        eprintln!(
            "Invalid start_url '{}' in kiosk configuration: {err}. Falling back to about:blank.",
            config.start_url
        );
        Url::parse("about:blank").expect("about:blank is a valid URL")
    });

    let start_url_for_window = start_url.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_fullscreen(true)?;
                window.navigate(start_url_for_window.clone())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

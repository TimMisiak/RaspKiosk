#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use anyhow::Context;
use config::{load_config, ConfigError, KioskConfig};
use tauri::WindowUrl;
use url::Url;

fn main() {
    if let Err(err) = run() {
        eprintln!("Application error: {err:?}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let config = match load_config() {
        Ok(config) => config,
        Err(ConfigError::NotFound) => {
            let fallback = KioskConfig::default();
            eprintln!(
                "kioskconfig.yaml not found; falling back to default URL {}",
                fallback.start_url
            );
            fallback
        }
        Err(other) => return Err(anyhow::Error::new(other)),
    };

    let url = Url::parse(&config.start_url)
        .with_context(|| format!("invalid start_url '{}'", config.start_url))?;

    let window_url = WindowUrl::External(url.clone());

    tauri::Builder::default()
        .setup(move |app| {
            tauri::WindowBuilder::new(app, "main", window_url.clone())
                .title("RaspKiosk")
                .fullscreen(true)
                .resizable(false)
                .visible(true)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|err| anyhow::Error::msg(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_url_is_reported() {
        let mut bad_config = KioskConfig::default();
        bad_config.start_url = "not a url".into();
        assert!(Url::parse(&bad_config.start_url).is_err());
    }
}

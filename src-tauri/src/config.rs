use serde::Deserialize;
use std::{env, fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RawKioskConfig {
    #[serde(default = "default_start_url")]
    pub start_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KioskConfig {
    pub start_url: String,
}

impl Default for KioskConfig {
    fn default() -> Self {
        Self {
            start_url: default_start_url(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unable to locate kioskconfig.yaml")]
    NotFound,
    #[error("failed to read kiosk configuration: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse kiosk configuration: {0}")]
    Parse(#[from] serde_yaml::Error),
}

pub fn load_config() -> Result<KioskConfig, ConfigError> {
    let path = resolve_config_path()?;
    let contents = fs::read_to_string(path)?;
    let raw: RawKioskConfig = serde_yaml::from_str(&contents)?;
    Ok(KioskConfig {
        start_url: raw.start_url,
    })
}

pub fn resolve_config_path() -> Result<PathBuf, ConfigError> {
    if let Some(from_env) = env::var_os("KIOSK_CONFIG") {
        let candidate = PathBuf::from(from_env);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    let cwd_candidate = env::current_dir()
        .map(|dir| dir.join("kioskconfig.yaml"))
        .ok();
    if let Some(candidate) = cwd_candidate {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let candidate = dir.join("kioskconfig.yaml");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    Err(ConfigError::NotFound)
}

fn default_start_url() -> String {
    "https://www.raspberrypi.com".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, io::Write};

    #[test]
    fn returns_default_when_missing_field() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("kioskconfig.yaml");
        fs::write(&path, "{}").expect("write config");

        env::set_var("KIOSK_CONFIG", &path);
        let config = load_config().expect("config");
        assert_eq!(config.start_url, default_start_url());
        env::remove_var("KIOSK_CONFIG");
    }

    #[test]
    fn prefers_env_path() {
        let dir = tempfile::tempdir().expect("temp dir");
        let env_path = dir.path().join("env.yaml");
        fs::write(&env_path, "start_url: https://example.com").expect("write config");
        env::set_var("KIOSK_CONFIG", &env_path);

        let cwd = tempfile::tempdir().expect("cwd");
        let old_cwd = env::current_dir().expect("cwd");
        env::set_current_dir(&cwd).expect("set cwd");
        fs::write(
            cwd.path().join("kioskconfig.yaml"),
            "start_url: https://fallback.com",
        )
        .expect("write fallback");

        let config = load_config().expect("config");
        assert_eq!(config.start_url, "https://example.com");

        env::set_current_dir(old_cwd).expect("restore cwd");
        env::remove_var("KIOSK_CONFIG");
    }

    #[test]
    fn returns_error_when_not_found() {
        env::remove_var("KIOSK_CONFIG");
        let cwd = env::current_dir().expect("cwd");
        let temp_dir = tempfile::tempdir().expect("temp dir");
        env::set_current_dir(temp_dir.path()).expect("set cwd");

        let result = resolve_config_path();
        env::set_current_dir(cwd).expect("restore");
        assert!(matches!(result, Err(ConfigError::NotFound)));
    }

    #[test]
    fn parse_valid_yaml() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("kioskconfig.yaml");
        let mut file = fs::File::create(&path).expect("file");
        writeln!(file, "start_url: https://tauri.app").expect("write");

        env::set_var("KIOSK_CONFIG", &path);
        let config = load_config().expect("config");
        assert_eq!(config.start_url, "https://tauri.app");
        env::remove_var("KIOSK_CONFIG");
    }
}

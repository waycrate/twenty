use duration_str::{deserialize_duration, parse};
use serde::Deserialize;
use std::{env, fs, io, path::PathBuf, time::Duration};

use crate::twenty_log;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_duration")]
    pub cooldown: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub lock_timer: Duration,
    pub theme: String,
    pub blacklisted: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cooldown: parse("20m").unwrap(),
            lock_timer: parse("20s").unwrap(),
            theme: "dark".to_string(),
            blacklisted: Vec::new(),
        }
    }
}

fn config_path() -> PathBuf {
    let base = env::var("XDG_CONFIG_HOME").unwrap_or({
        let home = env::var("HOME").unwrap_or(".".to_string());
        format!("{home}/.config")
    });
    PathBuf::from(base).join("twenty").join("config.toml")
}

pub fn load() -> Config {
    let path = config_path();

    if !path.exists() {
        let cfg = Config::default();
        let _ = cfg.save();
        twenty_log!("Created a default config at {}.", path.display());
    }

    match fs::read_to_string(&path) {
        Ok(text) => match toml::from_str(&text) {
            Ok(cfg) => cfg,
            Err(e) => {
                twenty_log!("Config file is invalid, using defaults. error: {}.", e);
                Config::default()
            }
        },
        Err(e) => {
            twenty_log!("Could not read config, using defaults. error: {}.", e);
            Config::default()
        }
    }
}

impl Config {
    pub fn save(&self) -> io::Result<()> {
        let path = config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        twenty_log!("Could not find ~/.config/twenty.toml, generating one.");
        let content =
            b"theme = \"dark\"\ncooldown = \"20m\"\nlock_timer = \"20s\"\nblacklisted = []";
        fs::write(path, content)?;
        Ok(())
    }
}

use std::fs;

use serde::Deserialize;

const CONFIG_BASENAME: &str = ".pm-spotlight";

#[derive(Clone, Deserialize)]
pub struct Config {
    pub search_paths: Vec<String>,
    pub skip_paths: Vec<String>,
}

pub struct ConfigManager {}

impl ConfigManager {
    pub fn load_configuration() -> Config {
        let config_filename = dirs::home_dir().unwrap().join(CONFIG_BASENAME); // `dirs` crate
        let config_str = fs::read_to_string(config_filename).unwrap();
        toml::from_str(&config_str).unwrap()
    }
}

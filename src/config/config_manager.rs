use std::fs;

use serde::Deserialize;

const CONFIG_BASENAME: &str = ".pm-spotlight";
const DEFAULT_CONFIG: &str = "search_paths = []\nskip_paths = []\n";

#[derive(Clone, Deserialize)]
pub struct Config {
    pub search_paths: Vec<String>,
    pub skip_paths: Vec<String>,
}

pub struct ConfigManager {}

impl ConfigManager {
    pub fn load_configuration() -> Result<Config, String> {
        let config_filename = dirs::home_dir()
            .ok_or_else(|| "Could not determine the home directory".to_string())?
            .join(CONFIG_BASENAME);

        let config_str = match fs::read_to_string(&config_filename) {
            Ok(config_str) => config_str,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::write(&config_filename, DEFAULT_CONFIG).map_err(|error| {
                    format!("Could not create {}: {error}", config_filename.display())
                })?;

                return Err(format!(
                    "Created {}. Add at least one entry to `search_paths` and restart",
                    config_filename.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "Could not read {}: {error}",
                    config_filename.display()
                ));
            }
        };

        let config: Config = toml::from_str(&config_str)
            .map_err(|error| format!("Could not parse {}: {error}", config_filename.display()))?;

        if config.search_paths.is_empty() {
            return Err(format!(
                "No search paths are configured in {}. Add at least one entry to `search_paths` and restart",
                config_filename.display()
            ));
        }

        Ok(config)
    }
}

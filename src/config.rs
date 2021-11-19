use crate::util::RumError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration file of Rum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the directory where game metadata and banners are stored
    pub data_path: PathBuf,
    /// Paths to scan when loading the library
    pub library_paths: Vec<PathBuf>,
    /// Terminal to use when opening a directory in the terminal
    pub terminal: String,
    /// List of scripts to ignore when scanning the library
    pub script_blacklist: Vec<String>,
}

impl Config {
    /// Creates a new Config struct by trying to load a configuration file located at:
    /// `$XDG_CONFIG_HOME/<APP NAME>/config.json`
    ///
    pub fn new(app_name: String) -> Self {
        let config_dir = dirs::config_dir().unwrap().join(&app_name);
        let _ = std::fs::create_dir_all(&config_dir);
        let config_file = config_dir.join("config.json");
        let config_data = std::fs::read_to_string(&config_file);
        if let Ok(config) = config_data.and_then(|data| {
            let json = serde_json::from_str(&data)?;
            Ok(json)
        }) {
            config
        } else {
            Config {
                data_path: dirs::data_dir().unwrap().join(&app_name),
                library_paths: vec![],
                terminal: "xterm".into(),
                script_blacklist: vec!["winetricks".into(), "rum.sh".into()],
            }
        }
    }

    /// Save the current configuration to the configuration file
    pub fn save(&self) -> Result<(), RumError> {
        let config_dir = dirs::config_dir().unwrap().join("rum");
        let _ = std::fs::create_dir_all(&config_dir);
        std::fs::write(
            &config_dir.join("config.json"),
            serde_json::to_string_pretty(&self)?,
        )?;
        Ok(())
    }

    pub fn data_path(&self) -> &Path {
        &self.data_path
    }

    pub fn library_paths(&self) -> &[PathBuf] {
        &self.library_paths
    }

    pub fn terminal(&self) -> &str {
        &self.terminal
    }

    pub fn set_data_path(&mut self, data_path: &Path) {
        self.data_path = data_path.into();
    }

    pub fn set_library_paths(&mut self, library_paths: &[PathBuf]) {
        self.library_paths = library_paths.into();
    }

    pub fn set_terminal(&mut self, terminal: &str) {
        self.terminal = terminal.into()
    }

    pub fn set_config(&mut self, other: Config) {
        self.data_path = other.data_path;
        self.library_paths = other.library_paths;
        self.terminal = other.terminal;
    }

}

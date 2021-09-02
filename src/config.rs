#[cfg(feature = "download")]
use crate::download::{TorrentClientConfig, TorrentConfig};

use crate::util::ChadError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration file of Chad Launcher
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Path to the directory where game metadata and banners are stored
    pub data_path: PathBuf,
    /// Paths to scan when loading the library
    pub library_paths: Vec<PathBuf>,
    /// Terminal to use when opening a directory in the terminal
    pub terminal: String,
    /// List of scripts to ignore when scanning the library
    pub script_blacklist: Vec<String>,
    /// Torrent client configuration
    #[cfg(feature = "download")]
    pub torrent: TorrentConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_path: dirs::data_dir().unwrap().join("chad_launcher"),
            library_paths: vec![],
            terminal: "alacritty".into(),
            script_blacklist: vec!["winetricks".into(), "chad.sh".into()],
            #[cfg(feature = "download")]
            torrent: TorrentConfig::default(),
        }
    }
}

impl Config {
    /// Creates a new Config struct by trying to load a configuuration file located at:
    /// `$XDG_CONFIG_HOME/chad_launcher/config.json`
    ///
    /// The location and file type of this file is subject to change in the near future.
    pub fn new() -> Self {
        let config_dir = dirs::config_dir().unwrap().join("chad_launcher");
        let _ = std::fs::create_dir_all(&config_dir);
        let config_file = config_dir.join("config.json");
        let config_data = std::fs::read_to_string(&config_file);
        if let Ok(config) = config_data.and_then(|data| {
            let json = serde_json::from_str(&data)?;
            Ok(json)
        }) {
            config
        } else {
            Self::default()
        }
    }

    /// Save the current configuration to the configuration file
    pub fn save(&self) -> Result<(), ChadError> {
        let config_dir = dirs::config_dir().unwrap().join("chad_launcher");
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

    #[cfg(feature = "download")]
    pub fn insert_download_client(
        &mut self,
        name: impl Into<String>,
        client_config: TorrentClientConfig,
    ) {
        self.torrent.clients.insert(name.into(), client_config);
    }

    #[cfg(feature = "download")]
    pub fn remove_download_client(&mut self, name: &str) {
        self.torrent.clients.remove(name);
    }
}

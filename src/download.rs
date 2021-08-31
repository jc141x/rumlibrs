use crate::{config::Config, util::ChadError};
use chad_torrent::{DelugeBackend, QBittorrentBackend, TorrentClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wrapper around [chad_torrent::Torrent](chad_torrent::Torrent) that adds a client field.
/// The torrent field is flattened when (de)serialized and the underlying [chad_torrent::Torrent](chad_torrent::Torrent)
/// is returned when dereferenced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub client: String,

    #[serde(flatten)]
    pub torrent: chad_torrent::Torrent,
}

impl std::ops::Deref for Torrent {
    type Target = chad_torrent::Torrent;

    fn deref(&self) -> &Self::Target {
        &self.torrent
    }
}

impl std::ops::DerefMut for Torrent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.torrent
    }
}

/// Configuration of a torrent client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend")]
pub enum TorrentClientConfig {
    #[serde(rename = "deluge")]
    Deluge(DelugeConfig),
    #[serde(rename = "qbittorrent")]
    QBittorrent(QBittorrentConfig),
}

/// Configuration of Deluge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelugeConfig {
    pub web_address: String,
    pub web_password: String,
    pub daemon_id: Option<String>,
}

/// Configuration of qBittorrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QBittorrentConfig {
    pub host: String,
    pub username: String,
    pub password: String,
}

#[derive(Default)]
pub struct DownloadManager {
    clients: HashMap<String, TorrentClient>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tries to connect to the clients configured in the given configuration.
    /// Will only attempt to connect when the torrent client is not already connected
    pub async fn load_config(&mut self, config: &Config) -> Result<(), ChadError> {
        for (name, c) in &config.torrent.clients {
            self.load_client(&name, &c).await.ok();
        }

        Ok(())
    }

    /// Tries to connect to the torrent client with the given name and configuration.
    /// Will only attempt to connect when the torrent client is not already connected
    pub async fn load_client(
        &mut self,
        name: &str,
        config: &TorrentClientConfig,
    ) -> Result<(), ChadError> {
        if !self.clients.contains_key(name) {
            self.clients.insert(
                name.into(),
                match config {
                    TorrentClientConfig::Deluge(options) => {
                        self.deluge_connect(&options).await.map(|c| c.into())
                    }
                    TorrentClientConfig::QBittorrent(options) => {
                        self.qbittorrent_connect(&options).await.map(|c| c.into())
                    }
                }?,
            );
        }
        Ok(())
    }

    /// Insert a client in the list of connected clients
    pub fn add_client(&mut self, name: &str, client: impl Into<TorrentClient>) {
        self.clients.insert(name.into(), client.into());
    }

    /// Remove a client from the list of connected clients
    pub fn remove_client(&mut self, name: &str) {
        self.clients.remove(name);
    }

    /// Try to connect to Deluge with the given configuration
    pub async fn deluge_connect(&self, config: &DelugeConfig) -> Result<DelugeBackend, ChadError> {
        let backend = DelugeBackend::new(&config.web_address, &config.web_password).await?;

        if let Some(daemon) = &config.daemon_id {
            backend.connect(daemon).await?;
        }

        Ok(backend)
    }

    /// Try to connect to qBittorrent with the given configuration
    pub async fn qbittorrent_connect(
        &self,
        config: &QBittorrentConfig,
    ) -> Result<QBittorrentBackend, ChadError> {
        Ok(QBittorrentBackend::new(&config.host, &config.username, &config.password).await?)
    }

    /// Returns a list of connected clients
    pub fn clients(&self) -> impl Iterator<Item = &String> {
        self.clients.keys()
    }

    /// Returns the connected client with the given name
    pub fn client(&self, name: &str) -> Option<&TorrentClient> {
        self.clients.get(name)
    }
}

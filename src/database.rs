use crate::util::ChadError;
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};

pub const API_KEY: &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoiYW5vbiIsImlhdCI6MTYyNzY0NDc0OCwiZXhwIjoxOTQzMjIwNzQ4fQ.MheXAiuWYFGDuFhfzAnANMzJU2UU4HN2dxwMxGdQd5A";

pub const TRACKERS: &[&'static str] = &[
    "udp://tracker.leechers-paradise.org:6969/announce",
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.zer0day.to:1337/announce",
    "udp://eddie4.nl:6969/announce",
    "udp://46.148.18.250:2710",
    "udp://opentor.org:2710",
    "http://tracker.dler.org:6969/announce",
    "udp://9.rarbg.me:2730/announce",
    "udp://9.rarbg.to:2770/announce",
    "udp://tracker.pirateparty.gr:6969/announce",
    "http://retracker.local/announce",
    "http://retracker.ip.ncnet.ru/announce",
    "udp://exodus.desync.com:6969/announce",
    "udp://ipv4.tracker.harry.lu:80/announce",
    "udp://open.stealth.si:80/announce",
    "udp://coppersurfer.tk:6969/announce",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    /// Unique identifier, primary key in the database
    pub id: usize,
    /// Id of the torrent on 1337x
    pub leetx_id: usize,
    /// Name of the game
    pub name: String,
    /// Version of the game
    pub version: String,
    /// Type: Wine or Native
    #[serde(rename = "type")]
    pub type_: String,
    /// Infohash of the torrent
    pub hash: String,
    /// Description of the game
    pub description: String,
    /// Whether the game is meant for a mature audience
    pub nsfw: bool,
    /// Relative path to the banner. Banners can be downloaded from here: `https://gitlab.com/chad-productions/chad_launcher_banners/-/raw/master/<banner_path>`
    pub banner_path: Option<String>,
    /// List of genres
    pub genres: Vec<String>,
    /// List of tags
    pub tags: Vec<String>,
    /// List of languages
    pub languages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGamesOpts {
    /// Page number starting from 0
    pub page_number: usize,
    /// Amount of games on each page
    pub page_size: usize,

    /// Language filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<String>,
    /// Tag filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_tag: Option<String>,
    /// Genre filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_genre: Option<String>,
    /// A search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ItemTable {
    Genres,
    Tags,
    Languages,
}

impl Into<&str> for ItemTable {
    fn into(self) -> &'static str {
        match self {
            Self::Genres => "genres",
            Self::Tags => "tags",
            Self::Languages => "languages",
        }
    }
}

pub struct DatabaseFetcher {
    client: Postgrest,
}

impl DatabaseFetcher {
    pub fn new() -> Self {
        Self {
            client: Postgrest::new("https://bkftwbhopivmrgzcagus.supabase.co/rest/v1/")
                .insert_header("apikey", API_KEY),
        }
    }

    /// Get a list of games from the database
    pub async fn get_games(&self, opts: &GetGamesOpts) -> Result<Vec<Game>, ChadError> {
        let result = self
            .client
            .rpc("get_games", &serde_json::to_string(&opts)?)
            .execute()
            .await?
            .json::<Vec<Game>>()
            .await?;

        Ok(result)
    }

    /// Gets a list of items from the given table name. For available table names, see
    /// [ItemTable](ItemTable).
    ///
    /// ```rust
    /// let genres = database.get_items(&self, ItemTable::Genres);
    /// ```
    pub async fn get_items(&self, table_name: impl Into<&str>) -> Result<Vec<String>, ChadError> {
        let result = self
            .client
            .rpc(table_name.into(), "")
            .execute()
            .await?
            .json::<Vec<String>>()
            .await?;

        Ok(result)
    }

    /// Find a banner for the given game name
    pub async fn find_banner(&self, game_name: &str) -> Result<String, ChadError> {
        let result = self
            .client
            .rpc("get_games", format!("{{ \"search\": \"{}\" }}", game_name))
            .execute()
            .await?
            .json::<Vec<Game>>()
            .await?;

        if let Some(game) = result.get(0) {
            if let Some(path) = &game.banner_path {
                Ok(path.into())
            } else {
                Err(ChadError::message("No banner found"))
            }
        } else {
            Err(ChadError::message("No game found"))
        }
    }
}

/// Returns a magnet link for the given game with the trackers in [TRACKERS](TRACKERS).
pub fn get_magnet(game: &Game) -> String {
    let mut magnet = format!("magnet:?xt=urn:btih:{}&dn={}", game.hash, game.name);
    for tracker in TRACKERS {
        magnet.push_str(&format!("&tr={}", tracker));
    }
    magnet
}

//! Database table documentation. The structs in this module match the internal database
//! structure.

use serde::{Deserialize, Serialize};

pub trait Table {
    fn table() -> &'static str;
}

pub trait Item: Table {
    fn new(game: &GameId, item: impl Into<String>) -> Self;
    fn field_name() -> &'static str;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub id: usize,
    pub origin: String,
    pub language: String,
}

impl Table for Language {
    fn table() -> &'static str {
        "language_v2"
    }
}

impl Item for Language {
    fn new(game: &GameId, item: impl Into<String>) -> Self {
        Self {
            id: game.id,
            origin: game.origin.clone(),
            language: item.into(),
        }
    }

    fn field_name() -> &'static str {
        "language"
    }
}

impl Into<String> for Language {
    fn into(self) -> String {
        self.language
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Genre {
    pub id: usize,
    pub origin: String,
    pub genre: String,
}

impl Table for Genre {
    fn table() -> &'static str {
        "genre_v2"
    }
}

impl Item for Genre {
    fn new(game: &GameId, item: impl Into<String>) -> Self {
        Self {
            id: game.id,
            origin: game.origin.clone(),
            genre: item.into(),
        }
    }

    fn field_name() -> &'static str {
        "genre"
    }
}

impl Into<String> for Genre {
    fn into(self) -> String {
        self.genre
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub id: usize,
    pub origin: String,
    pub tag: String,
}

impl Table for Tag {
    fn table() -> &'static str {
        "tag_v2"
    }
}

impl Item for Tag {
    fn new(game: &GameId, item: impl Into<String>) -> Self {
        Self {
            id: game.id,
            origin: game.origin.clone(),
            tag: item.into(),
        }
    }

    fn field_name() -> &'static str {
        "tag"
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.tag
    }
}

/// Primary key of game table
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameId {
    /// Id of the torrent on the origin (usually 1337x.to right now), PK
    pub id: usize,
    /// Origin, where does this game come from, PK
    pub origin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Game {
    /// Id of the torrent on the origin (usually 1337x.to right now), PK
    pub id: usize,
    /// Origin, where does this game come from, PK
    pub origin: String,
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
    /// Relative path to the banner. Banners can be downloaded from here: `https://gitlab.com/chad-productions/chad_launcher_banners/-/raw/master/<banner_rel_path>`
    pub banner_rel_path: Option<String>,
    /// Date on which the game was added to the database (not serialized!)
    #[serde(skip_serializing)]
    pub data_added: Option<String>,
}

impl Game {
    /// Returns the primary key of the game
    pub fn key(&self) -> GameId {
        GameId {
            id: self.id,
            origin: self.origin.clone(),
        }
    }
}

impl Table for Game {
    fn table() -> &'static str {
        "game_v2"
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListGames {
    /// Includes all fields from [Game](Game).
    /// Flattened during (de)serialization (see [Struct
    /// Flattening](https://serde.rs/attr-flatten.html)).
    /// [ListGames](ListGames) can also be dereferenced to access fields from [Game](Game).
    #[serde(flatten)]
    pub game: Game,
    /// List of genres
    pub genres: Vec<String>,
    /// List of tags
    pub tags: Vec<String>,
    /// List of languages
    pub languages: Vec<String>,
}

impl Table for ListGames {
    fn table() -> &'static str {
        "list_games_v2"
    }
}

impl std::ops::Deref for ListGames {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl std::ops::DerefMut for ListGames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListLanguages {
    language: String,
}

impl Table for ListLanguages {
    fn table() -> &'static str {
        "list_languages_v2"
    }
}

impl Into<String> for ListLanguages {
    fn into(self) -> String {
        self.language
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListGenres {
    genre: String,
}

impl Table for ListGenres {
    fn table() -> &'static str {
        "list_genres_v2"
    }
}

impl Into<String> for ListGenres {
    fn into(self) -> String {
        self.genre
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListTags {
    tag: String,
}

impl Table for ListTags {
    fn table() -> &'static str {
        "list_tags_v2"
    }
}

impl Into<String> for ListTags {
    fn into(self) -> String {
        self.tag
    }
}

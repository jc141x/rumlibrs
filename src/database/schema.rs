//! Database schema documentation. The structs in this module match the internal database
//! structure.

use serde::{Deserialize, Serialize};

pub trait Schema {
    fn table() -> &'static str;
}

/// Table: language
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language(Item);

impl std::ops::Deref for Language {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Language {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Schema for Language {
    fn table() -> &'static str {
        "language"
    }
}

impl Into<String> for Language {
    fn into(self) -> String {
        self.0.into()
    }
}

/// Table: genre
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Genre(Item);

impl std::ops::Deref for Genre {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Genre {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Schema for Genre {
    fn table() -> &'static str {
        "genre"
    }
}

impl Into<String> for Genre {
    fn into(self) -> String {
        self.0.into()
    }
}

/// Table: tag
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag(Item);

impl std::ops::Deref for Tag {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Tag {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Schema for Tag {
    fn table() -> &'static str {
        "tag"
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.0.into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: usize,
    pub name: String,
}

impl Into<String> for Item {
    fn into(self) -> String {
        self.name
    }
}

/// Table: game
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    /// Unique identifier, PK
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
    #[serde(rename = "banner_rel_path")]
    pub banner_path: Option<String>,
}

impl Schema for Game {
    fn table() -> &'static str {
        "game"
    }
}

/// Table: list_games (view)
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

impl Schema for ListGames {
    fn table() -> &'static str {
        "list_games"
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

/// Table: game_language
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameLanguage {
    /// PK, FK -> game.id
    pub game_id: usize,
    /// PK, FK -> language.id
    pub language_id: usize,
}

impl Schema for GameLanguage {
    fn table() -> &'static str {
        "game_language"
    }
}

/// Table: game_genre
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameGenre {
    /// PK, FK -> game.id
    pub game_id: usize,
    /// PK, FK -> genre.id
    pub genre_id: usize,
}

impl Schema for GameGenre {
    fn table() -> &'static str {
        "game_genre"
    }
}

/// Table: game_tag
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameTag {
    /// PK, FK -> game.id
    pub game_id: usize,
    /// PK, FK -> tag.id
    pub tag_id: usize,
}

impl Schema for GameTag {
    fn table() -> &'static str {
        "game_tag"
    }
}

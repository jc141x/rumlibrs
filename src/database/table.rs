//! Database table documentation. The structs in this module match the internal database
//! structure.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub trait Table {
    fn table() -> &'static str;
}

pub trait Item: Table {
    fn new(key: &GameKey<'_>, item: impl Into<String>) -> Self;
    fn field_name() -> &'static str;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub hash: String,
    pub file: String,
    pub language: String,
}

impl Table for Language {
    fn table() -> &'static str {
        "language_v4"
    }
}

impl Item for Language {
    fn new(key: &GameKey<'_>, item: impl Into<String>) -> Self {
        Self {
            hash: key.hash.into(),
            file: key.file.into(),
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
    pub hash: String,
    pub file: String,
    pub genre: String,
}

impl Table for Genre {
    fn table() -> &'static str {
        "genre_v4"
    }
}

impl Item for Genre {
    fn new(key: &GameKey<'_>, item: impl Into<String>) -> Self {
        Self {
            hash: key.hash.into(),
            file: key.file.into(),
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
    pub hash: String,
    pub file: String,
    pub tag: String,
}

impl Table for Tag {
    fn table() -> &'static str {
        "tag_v4"
    }
}

impl Item for Tag {
    fn new(key: &GameKey<'_>, item: impl Into<String>) -> Self {
        Self {
            hash: key.hash.into(),
            file: key.file.into(),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameKey<'a> {
    pub hash: &'a str,
    pub file: &'a str,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Game {
    /// Infohash of the torrent, PK
    pub hash: String,
    /// File to download, PK
    pub file: String,
    /// Name of the game
    pub name: String,
    /// Version of the game
    pub version: Option<String>,
    /// Description of the game
    pub description: String,
    /// Optional index for the banner
    pub banner_index: Option<usize>,
    /// Date on which the game was added to the database (not serialized!)
    #[serde(skip_serializing)]
    pub data_added: Option<String>,
    /// Id from 1337x, used for sorting
    pub leetx_id: usize,
}

impl Game {
    pub fn key<'a>(&'a self) -> GameKey<'a> {
        GameKey {
            hash: &self.hash,
            file: &self.file,
        }
    }
}

impl Table for Game {
    fn table() -> &'static str {
        "game_v4"
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ListGames {
    /// Includes all fields from [Game](Game).
    /// Flattened during (de)serialization (see [Struct
    /// Flattening](https://serde.rs/attr-flatten.html)).
    /// [ListGames](ListGames) can also be dereferenced to access fields from [Game](Game).
    #[serde(flatten)]
    pub game: Game,
    /// List of genres
    pub genres: BTreeSet<String>,
    /// List of tags
    pub tags: BTreeSet<String>,
    /// List of languages
    pub languages: BTreeSet<String>,
}

impl Table for ListGames {
    fn table() -> &'static str {
        "list_games_v4"
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

pub trait ItemList: Table + Into<String> {
    fn field_name() -> &'static str;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListLanguages {
    language: String,
}

impl Table for ListLanguages {
    fn table() -> &'static str {
        "list_languages_v4"
    }
}

impl ItemList for ListLanguages {
    fn field_name() -> &'static str {
        "language"
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
        "list_genres_v4"
    }
}

impl ItemList for ListGenres {
    fn field_name() -> &'static str {
        "genre"
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
        "list_tags_v4"
    }
}

impl ItemList for ListTags {
    fn field_name() -> &'static str {
        "tag"
    }
}

impl Into<String> for ListTags {
    fn into(self) -> String {
        self.tag
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestAuth {
    id: usize,
}

impl Table for TestAuth {
    fn table() -> &'static str {
        "test_auth"
    }
}

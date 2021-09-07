//! Database table documentation. The structs in this module match the internal database
//! structure.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub trait Table {
    fn table() -> &'static str;
}

pub trait Item: Table {
    fn new(hash: impl Into<String>, item: impl Into<String>) -> Self;
    fn field_name() -> &'static str;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub hash: String,
    pub language: String,
}

impl Table for Language {
    fn table() -> &'static str {
        "language_v3"
    }
}

impl Item for Language {
    fn new(hash: impl Into<String>, item: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
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
    pub genre: String,
}

impl Table for Genre {
    fn table() -> &'static str {
        "genre_v3"
    }
}

impl Item for Genre {
    fn new(hash: impl Into<String>, item: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
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
    pub tag: String,
}

impl Table for Tag {
    fn table() -> &'static str {
        "tag_v3"
    }
}

impl Item for Tag {
    fn new(hash: impl Into<String>, item: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
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

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Game {
    /// Infohash of the torrent, PK
    pub hash: String,
    /// Name of the game
    pub name: String,
    /// Version of the game
    pub version: String,
    /// Description of the game
    pub description: String,
    /// Relative path to the banner. Banners can be downloaded from here: `https://gitlab.com/chad-productions/chad_launcher_banners/-/raw/master/<banner_rel_path>`
    pub banner_rel_path: Option<String>,
    /// Date on which the game was added to the database (not serialized!)
    #[serde(skip_serializing)]
    pub data_added: Option<String>,
    /// Id from 1337x, used for sorting
    pub leetx_id: usize,
}

impl Table for Game {
    fn table() -> &'static str {
        "game_v3"
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
    pub genres: HashSet<String>,
    /// List of tags
    pub tags: HashSet<String>,
    /// List of languages
    pub languages: HashSet<String>,
}

impl Table for ListGames {
    fn table() -> &'static str {
        "list_games_v3"
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
        "list_languages_v3"
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
        "list_genres_v3"
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
        "list_tags_v3"
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

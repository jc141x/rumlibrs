//! Database schema documentation. The structs in this module match the internal database
//! structure.

use serde::{Deserialize, Serialize};

pub trait Schema {
    fn table() -> &'static str;
}

pub trait Item: Schema {
    fn new(game: &Game, item: impl Into<String>) -> Self;
    fn field_name() -> &'static str;
}

/// Table: language
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub id: usize,
    pub origin: String,
    pub language: String,
}

impl Schema for Language {
    fn table() -> &'static str {
        "language_v2"
    }
}

impl Item for Language {
    fn new(game: &Game, item: impl Into<String>) -> Self {
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

/// Table: genre
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Genre {
    pub id: usize,
    pub origin: String,
    pub genre: String,
}

impl Schema for Genre {
    fn table() -> &'static str {
        "genre_v2"
    }
}

impl Item for Genre {
    fn new(game: &Game, item: impl Into<String>) -> Self {
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

/// Table: tag
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub id: usize,
    pub origin: String,
    pub tag: String,
}

impl Schema for Tag {
    fn table() -> &'static str {
        "tag_v2"
    }
}

impl Item for Tag {
    fn new(game: &Game, item: impl Into<String>) -> Self {
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

/// Table: game
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
    /// Relative path to the banner. Banners can be downloaded from here: `https://gitlab.com/chad-productions/chad_launcher_banners/-/raw/master/<banner_path>`
    #[serde(rename = "banner_rel_path")]
    pub banner_path: Option<String>,
    /// Date on which the game was added to the database (not serialized!)
    #[serde(skip_serializing)]
    pub data_added: Option<String>,
}

impl Schema for Game {
    fn table() -> &'static str {
        "game_v2"
    }
}

/// Table: list_games (view)
///
/// Query for reference:
///
/// ```postgresql
/// create view list_games as select
///     game.id,
///     game.leetx_id,
///     game.name,
///     game.version,
///     game.type,
///     game.hash,
///     game.description,
///     game.nsfw,
///     game.banner_rel_path,
///     array_remove(array_agg(distinct genre.name), null) genres,
///     array_remove(array_agg(distinct tag.name), null) tags,
///     array_remove(array_agg(distinct language.name), null) languages
///   from game
///   left outer join game_genre on game.id = game_genre.game_id
///   left outer join game_tag on game.id = game_tag.game_id
///   left outer join game_language on game.id = game_language.game_id
///   left outer join genre on genre.id = game_genre.genre_id
///   left outer join tag on tag.id = game_tag.tag_id
///   left outer join language on language.id = game_language.language_id
///   group by (game.id)
///   order by game.leetx_id desc
/// ```
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

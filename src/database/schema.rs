//! Database schema documentation. The structs in this module match the internal database
//! structure.

use serde::{Deserialize, Serialize};

pub trait Schema {
    fn table() -> &'static str;
}

/// Table: language
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub id: usize,
    pub name: String,
}

impl Item for Language {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Schema for Language {
    fn table() -> &'static str {
        "language"
    }
}

impl Into<String> for Language {
    fn into(self) -> String {
        self.name
    }
}

/// Table: genre
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Genre {
    pub id: usize,
    pub name: String,
}

impl Item for Genre {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Schema for Genre {
    fn table() -> &'static str {
        "genre"
    }
}

impl Into<String> for Genre {
    fn into(self) -> String {
        self.name
    }
}

/// Table: tag
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub id: usize,
    pub name: String,
}

impl Item for Tag {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Schema for Tag {
    fn table() -> &'static str {
        "tag"
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.name
    }
}

pub trait Item: Schema {
    fn id(&self) -> usize;
    fn name(&self) -> &str;
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

pub trait Junction2: Schema {
    fn new(first: usize, second: usize) -> Self;
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

impl Junction2 for GameLanguage {
    fn new(first: usize, second: usize) -> Self {
        Self {
            game_id: first,
            language_id: second,
        }
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

impl Junction2 for GameGenre {
    fn new(first: usize, second: usize) -> Self {
        Self {
            game_id: first,
            genre_id: second,
        }
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

impl Junction2 for GameTag {
    fn new(first: usize, second: usize) -> Self {
        Self {
            game_id: first,
            tag_id: second,
        }
    }
}

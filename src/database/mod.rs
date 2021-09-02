pub mod table;
pub use table::ListGames as Game;

use crate::util::ChadError;
use async_trait::async_trait;
use futures::try_join;
use postgrest::Postgrest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// johncena141 supabase PostgREST endpoint
pub const SUPABASE_ENDPOINT: &'static str = "https://bkftwbhopivmrgzcagus.supabase.co/rest/v1";

/// johncena141 database publick API key
pub const SUPABASE_PUBLIC_API_KEY: &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoiYW5vbiIsImlhdCI6MTYyNzY0NDc0OCwiZXhwIjoxOTQzMjIwNzQ4fQ.MheXAiuWYFGDuFhfzAnANMzJU2UU4HN2dxwMxGdQd5A";

/// List of trackers used by johncena141 torrents
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetGamesOpts {
    /// Page number starting from 0
    pub page_number: usize,
    /// Amount of games on each page
    pub page_size: usize,

    /// Language filter
    pub filter_languages: Vec<String>,
    /// Tag filter
    pub filter_tags: Vec<String>,
    /// Genre filter
    pub filter_genres: Vec<String>,
    /// A search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

pub struct DatabaseFetcher {
    client: Postgrest,
}

#[async_trait]
pub trait BuilderExt {
    /// Like execute but checks error code
    async fn run(self) -> Result<reqwest::Response, ChadError>;
    /// Shorthand for `self.run().await?.json().await`
    async fn json<T: DeserializeOwned>(self) -> Result<T, ChadError>;
}

#[async_trait]
impl BuilderExt for postgrest::Builder {
    async fn run(self) -> Result<reqwest::Response, ChadError> {
        let res = self.execute().await?;

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(ChadError::DatabaseError(res.status().as_u16().into()))
        }
    }

    async fn json<T: DeserializeOwned>(self) -> Result<T, ChadError> {
        Ok(self.run().await?.json().await?)
    }
}

impl DatabaseFetcher {
    /// Create a new DatabaseFetcher using the given supabase endpoint and supabase API key.
    pub fn new(endpoint: &str, api_key: &str) -> Self {
        Self {
            client: Postgrest::new(endpoint)
                .insert_header("apikey", api_key)
                .insert_header("Authorization", format!("Bearer {}", api_key)),
        }
    }

    /// Uses johncena141 database endpoint and public API key by default.
    pub fn default() -> Self {
        Self::new(SUPABASE_ENDPOINT, SUPABASE_PUBLIC_API_KEY)
    }

    /// Creates a new query builder
    ///
    /// ```rust
    /// # use chad_rs::database::DatabaseFetcher;
    /// use chad_rs::database::BuilderExt;
    /// use chad_rs::database::table;
    ///
    /// # tokio_test::block_on(async {
    /// # let database = DatabaseFetcher::default();
    /// let genres: Vec<table::ListGenres> = database.from::<table::ListGenres>().json().await.unwrap();
    /// # });
    /// ```
    pub fn from<T: table::Table>(&self) -> postgrest::Builder {
        self.client.from(T::table())
    }

    /// Lists a table in the database
    ///
    /// ```rust
    /// # use chad_rs::database::DatabaseFetcher;
    /// use chad_rs::database::table;
    ///
    /// # tokio_test::block_on(async {
    /// # let database = DatabaseFetcher::default();
    /// let games: Vec<table::Game> = database.list_table().await.unwrap();
    /// let languages: Vec<String> = database.list_table::<table::ListLanguages>().await.unwrap().into_iter().map(|l| l.into()).collect();
    /// # });
    /// ```
    pub async fn list_table<T: table::Table + DeserializeOwned>(
        &self,
    ) -> Result<Vec<T>, ChadError> {
        self.from::<T>().select("*").json().await
    }

    /// Get a list of games from the database
    ///
    /// ```rust
    /// use chad_rs::database::GetGamesOpts;
    /// # use chad_rs::database::DatabaseFetcher;
    ///
    /// # let database = DatabaseFetcher::default();
    /// let opts = GetGamesOpts {
    ///     page_number: 0,
    ///     page_size: 20,
    ///     filter_genres: vec!["Action".into()],
    ///     ..Default::default()
    /// };
    /// # tokio_test::block_on(async {
    /// let res = database.get_games(&opts).await.unwrap();
    /// assert_eq!(res.len(), 20);
    /// #    println!("{:#?}", res);
    /// # });
    /// ```
    pub async fn get_games(&self, opts: &GetGamesOpts) -> Result<Vec<Game>, ChadError> {
        let mut builder = self.from::<table::ListGames>().select("*").range(
            opts.page_number,
            opts.page_number * opts.page_size + opts.page_size - 1,
        );

        if !opts.filter_languages.is_empty() {
            builder = builder.ov(
                "languages",
                format!("{{{}}}", opts.filter_languages.join(",")),
            )
        }

        if !opts.filter_genres.is_empty() {
            builder = builder.ov("genres", format!("{{{}}}", opts.filter_genres.join(",")))
        }

        if !opts.filter_tags.is_empty() {
            builder = builder.ov("tags", format!("{{{}}}", opts.filter_tags.join(",")))
        }

        if let Some(query) = &opts.search {
            builder = builder.ilike("name", format!("*{}*", query))
        }

        builder.json().await
    }

    /// Find a banner for the given game name
    ///
    /// ```rust
    /// # use chad_rs::database::DatabaseFetcher;
    ///
    /// # let database = DatabaseFetcher::default();
    /// # tokio_test::block_on(async {
    /// let banner = database.find_banner("Minecraft").await;
    /// # });
    /// ```
    pub async fn find_banner(&self, game_name: &str) -> Result<String, ChadError> {
        let result = self
            .from::<table::Game>()
            .select("*")
            .ilike("name", format!("{}", game_name))
            .json::<Vec<table::Game>>()
            .await?;

        if let Some(game) = result.get(0) {
            if let Some(path) = &game.banner_rel_path {
                Ok(path.into())
            } else {
                Err(ChadError::message("No banner found"))
            }
        } else {
            Err(ChadError::message("No game found"))
        }
    }

    /// Upsert a row into a table
    pub async fn upsert<T: table::Table + Serialize>(&self, item: &T) -> Result<(), ChadError> {
        self.from::<T>()
            .upsert(serde_json::to_string(item)?)
            .run()
            .await?;
        Ok(())
    }

    /// Insert a row into a table
    pub async fn insert<T: table::Table, V: Serialize>(&self, item: &V) -> Result<(), ChadError> {
        self.from::<T>()
            .insert(serde_json::to_string(item)?)
            .run()
            .await?;
        Ok(())
    }

    /// Upsert all rows into a table
    pub async fn upsert_all<T: table::Table, V: Serialize>(
        &self,
        items: &[V],
    ) -> Result<(), ChadError> {
        self.from::<T>()
            .upsert(serde_json::to_string(items)?)
            .run()
            .await?;
        Ok(())
    }

    /// Insert all rows into a table
    pub async fn insert_all<T: table::Table, V: Serialize>(
        &self,
        items: &[V],
    ) -> Result<(), ChadError> {
        self.from::<T>()
            .insert(serde_json::to_string(items)?)
            .run()
            .await?;
        Ok(())
    }

    /// Add items ([Item](table::Item)) for the given game to the table
    pub async fn add_items<I>(
        &self,
        game_id: &table::GameId,
        items: &[String],
    ) -> Result<(), ChadError>
    where
        I: table::Item + Serialize,
    {
        let items = items
            .iter()
            .map(|item| I::new(game_id, item))
            .collect::<Vec<_>>();
        self.upsert_all::<I, _>(&items).await
    }

    /// Delete items ([Item](table::Item)) for the given game from the table
    pub async fn delete_items<I>(
        &self,
        game_id: &table::GameId,
        items: &[String],
    ) -> Result<(), ChadError>
    where
        I: table::Item + Serialize,
    {
        self.from::<I>()
            .and(format!(
                "id.eq.{},origin.eq.{},{}.in.({})",
                game_id.id,
                &game_id.origin,
                I::field_name(),
                items.join(",")
            ))
            .delete()
            .run()
            .await?;
        Ok(())
    }

    /// Delete all rows that match with the given game_id from a table
    pub async fn delete_game_from<T>(&self, game_id: &table::GameId) -> Result<(), ChadError>
    where
        T: table::Table,
    {
        self.from::<T>()
            .and(format!(
                "id.eq.{},origin.eq.{}",
                game_id.id, &game_id.origin,
            ))
            .delete()
            .run()
            .await?;
        Ok(())
    }

    /// Add or update a game to the database with the given languages, genres and tags
    pub async fn add_update_game(
        &self,
        game: &table::Game,
        languages: &[String],
        genres: &[String],
        tags: &[String],
    ) -> Result<(), ChadError> {
        self.upsert::<table::Game>(game).await?;
        let key = game.key();

        try_join!(
            self.add_items::<table::Language>(&key, languages),
            self.add_items::<table::Genre>(&key, genres),
            self.add_items::<table::Tag>(&key, tags),
        )?;

        Ok(())
    }

    /// Remove a game from the database. Also removes all its entries from language, genre and tag
    /// tables.
    ///
    /// This function does nothing more than call delete_game_from on each database table.
    pub async fn remove_game(&self, game_id: &table::GameId) -> Result<(), ChadError> {
        try_join!(
            self.delete_game_from::<table::Language>(game_id),
            self.delete_game_from::<table::Genre>(game_id),
            self.delete_game_from::<table::Tag>(game_id),
        )?;
        self.delete_game_from::<table::Game>(game_id).await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_game_remove_languages_and_remove() {
        use table;

        if let Ok(key) = std::env::var("SUPABASE_SECRET_KEY") {
            let database = DatabaseFetcher::new(SUPABASE_ENDPOINT, &key);

            let game = table::Game {
                id: 1337,
                name: "Hello there".into(),
                origin: "your mom".into(),
                description: "I'm testing the insertion of new games into the database".into(),
                hash: "This is not a valid infohash at all".into(),
                nsfw: true,
                type_: "Native".into(),
                version: "Version".into(),
                ..Default::default()
            };

            let languages = &[
                "Klingon".into(),
                "Vulcan".into(),
                "Dothraki".into(),
                "Trigedasleng".into(),
            ];
            let genres = &["Impossible".into(), "Fake".into(), "Test".into()];
            let tags = &["Send".into(), "Help".into()];

            database
                .add_update_game(&game, languages, genres, tags)
                .await
                .unwrap();

            database
                .delete_items::<table::Language>(
                    &game.key(),
                    &["Klingon".into(), "Vulcan".into(), "aaa".into()],
                )
                .await
                .unwrap();

            database.remove_game(&game.key()).await.unwrap();
            database
                .remove_game(&table::GameId {
                    id: 1337,
                    origin: "leetx".into(),
                })
                .await
                .unwrap();
        } else {
            println!("Supabase admin key not set, skipping test")
        }
    }
}

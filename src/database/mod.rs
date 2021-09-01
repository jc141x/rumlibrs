pub mod schema;
pub use schema::ListGames as Game;

use crate::util::ChadError;
use async_trait::async_trait;
use futures::future::try_join_all;
use futures::try_join;
use postgrest::Postgrest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

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
    /// Shorthand for `self.execute().await?.json().await`
    async fn json<T: DeserializeOwned>(self) -> Result<T, reqwest::Error>;
}

#[async_trait]
impl BuilderExt for postgrest::Builder {
    async fn json<T: DeserializeOwned>(self) -> Result<T, reqwest::Error> {
        self.execute().await?.json().await
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
    /// use chad_rs::database::schema;
    ///
    /// # tokio_test::block_on(async {
    /// # let database = DatabaseFetcher::default();
    /// let genres: Vec<schema::Genre> = database.from::<schema::Genre>().json().await.unwrap();
    /// # });
    /// ```
    pub fn from<T: schema::Schema + DeserializeOwned>(&self) -> postgrest::Builder {
        self.client.from(T::table())
    }

    /// Lists a table in the database
    ///
    /// ```rust
    /// # use chad_rs::database::DatabaseFetcher;
    /// use chad_rs::database::schema;
    ///
    /// # tokio_test::block_on(async {
    /// # let database = DatabaseFetcher::default();
    /// let games: Vec<schema::Game> = database.list_table().await.unwrap();
    /// let languages: Vec<String> = database.list_table::<schema::Language>().await.unwrap().into_iter().map(|l| l.into()).collect();
    /// # });
    /// ```
    pub async fn list_table<T: schema::Schema + DeserializeOwned>(
        &self,
    ) -> Result<Vec<T>, ChadError> {
        Ok(self.from::<T>().select("*").json().await?)
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
    ///     filter_languages: vec!["Latin".into(), "Dutch".into()],
    ///     filter_genres: vec!["Action".into(), "Adventure".into()],
    ///     ..Default::default()
    /// };
    /// # tokio_test::block_on(async {
    /// let res = database.get_games(&opts).await.unwrap();
    /// #    println!("{:#?}", res);
    /// # });
    /// ```
    pub async fn get_games(&self, opts: &GetGamesOpts) -> Result<Vec<Game>, ChadError> {
        let mut builder = self
            .from::<schema::ListGames>()
            .select("*")
            .range(opts.page_number, opts.page_number + opts.page_size - 1);

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

        let result = builder.json().await?;

        Ok(result)
    }

    /// Gets a list of items from the given table name. For available table names, see
    /// [ItemTable](ItemTable).
    ///
    /// ```rust
    /// use chad_rs::database::ItemTable;
    /// # use chad_rs::database::DatabaseFetcher;
    ///
    /// # let database = DatabaseFetcher::default();
    /// # tokio_test::block_on(async {
    /// let genres = database.get_items(ItemTable::Genres).await;
    /// # })
    /// ```
    #[deprecated(since = "0.2.0", note = "please use `list_table` or `from` instead")]
    pub async fn get_items(&self, _table_name: impl Into<&str>) -> Result<Vec<String>, ChadError> {
        unimplemented!()
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
            .from::<schema::Game>()
            .select("*")
            .ilike("name", format!("{}", game_name))
            .json::<Vec<schema::Game>>()
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

    pub async fn upsert<T: schema::Schema + Serialize>(&self, item: &T) -> Result<(), ChadError> {
        self.client
            .from(T::table())
            .upsert(serde_json::to_string(item)?)
            .execute()
            .await?;
        Ok(())
    }

    pub async fn insert<T: schema::Schema, V: Serialize>(&self, item: &V) -> Result<(), ChadError> {
        self.client
            .from(T::table())
            .insert(serde_json::to_string(item)?)
            .execute()
            .await?;
        Ok(())
    }

    pub async fn insert_all<T: schema::Schema, V: Serialize + std::fmt::Debug>(
        &self,
        items: &[V],
    ) -> Result<(), ChadError> {
        println!("insert_all: {:#?}", items);
        let res = self
            .client
            .from(T::table())
            .insert(serde_json::to_string(items)?)
            .execute()
            .await?;
        println!("{:#?}", res);
        Ok(())
    }

    pub async fn upsert_all<T: schema::Schema, V: Serialize + std::fmt::Debug>(
        &self,
        items: &[V],
    ) -> Result<(), ChadError> {
        println!("upsert_all: {:#?}", items);
        let res = self
            .client
            .from(T::table())
            .upsert(serde_json::to_string(items)?)
            .execute()
            .await?;
        println!("{:#?}", res);
        Ok(())
    }

    /// Try to insert a new [Item](schema::Item) in a table. If the item already exists, this
    /// function will still succeed.
    ///
    /// ```rust
    /// # use chad_rs::database::DatabaseFetcher;
    /// use chad_rs::database::schema;
    ///
    /// # let database = DatabaseFetcher::default();
    /// # tokio_test::block_on(async {
    /// database.insert_item::<schema::Language>("English").await.unwrap();
    /// # });
    ///
    /// ```
    pub async fn insert_item<T: schema::Item + DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<usize, ChadError> {
        println!("Inserting item {} into {} ...", name, T::table());

        let json = json!({ "name": name });

        let res = self
            .client
            .from(T::table())
            .insert(serde_json::to_string(&json)?)
            .execute()
            .await?;

        let status = res.status().as_u16();

        if status == 409 || status == 201 {
            Ok(self
                .find_rows_with::<T>("name", &name)
                .await?
                .get(0)
                .ok_or(ChadError::Message(format!(
                    "Failed to add item {} to {}",
                    name,
                    T::table()
                )))?
                .id())
        } else {
            Err(ChadError::DatabaseError(status))
        }
    }

    pub async fn insert_items<T: schema::Item + DeserializeOwned>(
        &self,
        names: &[String],
    ) -> Result<(), ChadError> {
        println!("Bulk inserting items {:?} into {} ...", names, T::table());

        //let db_items = self.get_items_from_names::<T>(names).await?;

        let json: Vec<_> = names.iter().map(|name| json!({ "name": name })).collect();

        let res = self
            .client
            .from(T::table())
            .insert(serde_json::to_string(&json)?)
            .execute()
            .await?;

        let status = res.status().as_u16();

        if status == 409 || status == 201 {
            Ok(())
        } else {
            Err(ChadError::DatabaseError(status))
        }
    }

    pub async fn get_items_from_names<T: schema::Item + DeserializeOwned>(
        &self,
        names: &[String],
    ) -> Result<Vec<T>, ChadError> {
        Ok(self
            .from::<T>()
            .select("*")
            .in_("name", names)
            .json()
            .await?)
    }

    pub async fn insert_game_items<I, J>(
        &self,
        game_id: usize,
        items: &[String],
    ) -> Result<(), ChadError>
    where
        I: schema::Item + DeserializeOwned + std::fmt::Debug,
        J: schema::Junction2 + Serialize + std::fmt::Debug,
    {
        self.insert_items::<I>(items).await?;
        let items = self.get_items_from_names::<I>(items).await?;
        println!("{:#?}", &items);
        let res = self
            .upsert_all::<J, _>(
                &items
                    .iter()
                    .map(schema::Item::id)
                    .map(|item_id| J::new(game_id, item_id))
                    .collect::<Vec<_>>(),
            )
            .await?;
        println!("{:#?}", res);
        Ok(())
    }

    pub async fn add_game(
        &self,
        game: &schema::Game,
        languages: &[String],
        genres: &[String],
        tags: &[String],
    ) -> Result<(), ChadError> {
        let leetx_id = game.leetx_id.to_string();
        let mut game_obj: serde_json::Value = serde_json::to_value(&game)?;

        // Remove the id in order to use auto increment
        game_obj
            .as_object_mut()
            .ok_or(ChadError::Message("Invalid game".into()))?
            .remove("id");

        let mut same_games = self
            .find_rows_with::<schema::Game>("leetx_id", &leetx_id)
            .await?;

        if let Some(db_game) = same_games.pop() {
            let mut new_game = game.clone();
            new_game.id = db_game.id;
            println!("Game already in database, updating ...");
            self.upsert::<schema::Game>(&new_game).await?;
        } else {
            println!("Inserting game...");
            self.insert::<schema::Game, serde_json::Value>(&game_obj)
                .await?;
        }

        let game_id = self
            .find_rows_with::<schema::Game>("leetx_id", &leetx_id)
            .await?
            .get(0)
            .ok_or(ChadError::Message("Failed to add game to database".into()))?
            .id;

        let language_future =
            self.insert_game_items::<schema::Language, schema::GameLanguage>(game_id, languages);
        let genre_future =
            self.insert_game_items::<schema::Genre, schema::GameGenre>(game_id, genres);
        let tag_future = self.insert_game_items::<schema::Tag, schema::GameTag>(game_id, tags);
        /*
        let language_future = async {
            self.insert_items::<schema::Language>(&languages).await?;
            let items = self
                .get_items_from_names::<schema::Language>(&languages)
                .await?;
            self.insert_all::<schema::GameLanguage, _>(
                &items
                    .iter()
                    .map(schema::Item::id)
                    .map(|language_id| schema::GameLanguage {
                        game_id,
                        language_id,
                    })
                    .collect::<Vec<_>>(),
            )
            .await
        };

        let language_futures = try_join_all(languages.into_iter().map(|l| async move {
            let id = self.insert_item::<schema::Language>(&l).await?;
            println!("Inserting ({}, {}) into game_language ...", game_id, id);
            self.insert::<schema::GameLanguage, schema::GameLanguage>(&schema::GameLanguage {
                game_id,
                language_id: id,
            })
            .await
        }));

        let genre_futures = try_join_all(genres.into_iter().map(|g| async move {
            let id = self.insert_item::<schema::Genre>(&g).await?;
            println!("Inserting ({}, {}) into game_genre ...", game_id, id);
            self.insert::<schema::GameGenre, schema::GameGenre>(&schema::GameGenre {
                game_id,
                genre_id: id,
            })
            .await
        }));

        let tag_futures = try_join_all(tags.into_iter().map(|g| async move {
            let id = self.insert_item::<schema::Tag>(&g).await?;
            println!("Inserting ({}, {}) into game_tag ...", game_id, id);
            self.insert::<schema::GameTag, schema::GameTag>(&schema::GameTag {
                game_id,
                tag_id: id,
            })
            .await
        }));*/

        try_join!(language_future, genre_future, tag_future)?;

        println!("Done");

        Ok(())
    }

    pub async fn find_rows_with<T: schema::Schema + DeserializeOwned>(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<T>, ChadError> {
        Ok(self
            .from::<T>()
            .select("*")
            .eq(key, value)
            .json::<Vec<T>>()
            .await?)
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
    async fn test_insert_item() {
        use schema;

        let database = DatabaseFetcher::new(
            SUPABASE_ENDPOINT,
            &std::env::var("SUPABASE_SECRET_KEY").expect("Please set your supabase secret key"),
        );
        println!(
            "{:#?}",
            database.insert_item::<schema::Language>("Test123").await
        );
    }

    #[tokio::test]
    async fn test_add_game() {
        use schema;

        let database = DatabaseFetcher::new(
            SUPABASE_ENDPOINT,
            &std::env::var("SUPABASE_SECRET_KEY").expect("Please set your supabase secret key"),
        );

        let game = schema::Game {
            id: 1,
            name: "Test Game Please Ignore Me".into(),
            banner_path: None,
            description: "I'm testing the insertion of new games into the database".into(),
            hash: "This is not a valid infohash at all".into(),
            leetx_id: 1337,
            nsfw: true,
            type_: "Native".into(),
            version: "Version".into(),
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
            .add_game(&game, languages, genres, tags)
            .await
            .unwrap();
    }
}

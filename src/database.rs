pub use crate::schema::ListGames as Game;

use crate::schema;
use crate::util::ChadError;
use async_trait::async_trait;
use postgrest::Postgrest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const SUPABASE_ENDPOINT: &'static str = "https://bkftwbhopivmrgzcagus.supabase.co/rest/v1/";
pub const SUPABASE_PUBLIC_API_KEY: &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoiYW5vbiIsImlhdCI6MTYyNzY0NDc0OCwiZXhwIjoxOTQzMjIwNzQ4fQ.MheXAiuWYFGDuFhfzAnANMzJU2UU4HN2dxwMxGdQd5A";

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
            client: Postgrest::new(endpoint).insert_header("apikey", api_key),
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
    /// use chad_rs::schema;
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
    /// use chad_rs::schema;
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
    pub async fn get_items(&self, table_name: impl Into<&str>) -> Result<Vec<String>, ChadError> {
        let result = self
            .client
            .from(table_name.into())
            .json::<Vec<schema::Item>>()
            .await?
            .into_iter()
            .map(|v| v.name)
            .collect();

        Ok(result)
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
}

/// Returns a magnet link for the given game with the trackers in [TRACKERS](TRACKERS).
pub fn get_magnet(game: &Game) -> String {
    let mut magnet = format!("magnet:?xt=urn:btih:{}&dn={}", game.hash, game.name);
    for tracker in TRACKERS {
        magnet.push_str(&format!("&tr={}", tracker));
    }
    magnet
}

pub use crate::schema::ListGames as Game;

use crate::schema;
use crate::util::ChadError;
use postgrest::Postgrest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const PUBLIC_API_KEY: &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoiYW5vbiIsImlhdCI6MTYyNzY0NDc0OCwiZXhwIjoxOTQzMjIwNzQ4fQ.MheXAiuWYFGDuFhfzAnANMzJU2UU4HN2dxwMxGdQd5A";

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
    pub filter_language: Option<String>,
    /// Tag filter
    pub filter_tag: Option<String>,
    /// Genre filter
    pub filter_genre: Option<String>,
    /// A search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

pub struct Builder<T: schema::Schema + DeserializeOwned> {
    _schema: std::marker::PhantomData<T>,
    builder: postgrest::Builder,
}

impl<S: schema::Schema + DeserializeOwned> Builder<S> {
    pub fn new(client: &Postgrest) -> Self {
        Self {
            _schema: std::marker::PhantomData,
            builder: client.from(S::table()),
        }
    }

    pub fn select<T>(mut self, columns: T) -> Self
    where
        T: Into<String>,
    {
        self.builder = self.builder.select(columns);
        self
    }

    pub async fn execute(self) -> Result<Vec<S>, ChadError> {
        Ok(self.builder.execute().await?.json().await?)
    }
}

pub struct DatabaseFetcher {
    client: Postgrest,
}

impl DatabaseFetcher {
    pub fn new(api_key: Option<&str>) -> Self {
        Self {
            client: Postgrest::new("https://bkftwbhopivmrgzcagus.supabase.co/rest/v1/")
                .insert_header("apikey", api_key.unwrap_or(PUBLIC_API_KEY)),
        }
    }

    /// Creates a new query builder
    ///
    /// ```rust
    /// use chad_rs::database::DatabaseFetcher;
    /// use chad_rs::schema;
    ///
    /// async {
    ///     let database = DatabaseFetcher::new(None);
    ///     let genres: Vec<schema::Genre> = database.from().execute().await.unwrap();
    /// };
    /// ```
    pub fn from<T: schema::Schema + DeserializeOwned>(&self) -> Builder<T> {
        Builder::new(&self.client)
    }

    /// Lists a table in the database
    ///
    /// ```rust
    /// use chad_rs::database::DatabaseFetcher;
    /// use chad_rs::schema;
    ///
    /// async {
    ///     let database = DatabaseFetcher::new(None);
    ///     let genres: Vec<schema::Game> = database.list_table().await.unwrap();
    /// };
    /// ```
    pub async fn list_table<T: schema::Schema + DeserializeOwned>(
        &self,
    ) -> Result<Vec<T>, ChadError> {
        Builder::new(&self.client).select("*").execute().await
    }

    /// Get a list of games from the database
    pub async fn get_games(&self, opts: &GetGamesOpts) -> Result<Vec<Game>, ChadError> {
        let mut builder = self
            .client
            .from("list_games")
            .select("*")
            .range(opts.page_number, opts.page_number + opts.page_size - 1);

        if let Some(language) = &opts.filter_language {
            builder = builder.cs("languages", format!("{{{}}}", language))
        }

        if let Some(genre) = &opts.filter_genre {
            builder = builder.cs("genres", format!("{{{}}}", genre))
        }

        if let Some(tag) = &opts.filter_tag {
            builder = builder.cs("tags", format!("{{{}}}", tag))
        }

        if let Some(query) = &opts.search {
            builder = builder.ilike("name", format!("*{}*", query))
        }

        let result = builder.execute().await?.json().await?;

        Ok(result)
    }

    /// Gets a list of items from the given table name. For available table names, see
    /// [ItemTable](ItemTable).
    ///
    /// ```rust
    /// use chad_rs::database::{DatabaseFetcher, ItemTable};
    ///
    /// let database = DatabaseFetcher::new(None);
    /// let genres = database.get_items(ItemTable::Genres);
    /// ```
    #[deprecated(since = "0.2.0", note = "please use `list_table` or `from` instead")]
    pub async fn get_items(&self, table_name: impl Into<&str>) -> Result<Vec<String>, ChadError> {
        let result = self
            .client
            .from(table_name.into())
            .execute()
            .await?
            .json::<Vec<schema::Item>>()
            .await?
            .into_iter()
            .map(|v| v.name)
            .collect();

        Ok(result)
    }

    /// Find a banner for the given game name
    pub async fn find_banner(&self, game_name: &str) -> Result<String, ChadError> {
        let result = self
            .client
            .from("game")
            .select("*")
            .ilike("name", format!("{}", game_name))
            .execute()
            .await?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_games() -> Result<(), ChadError> {
        let database = DatabaseFetcher::new(None);
        let opts = GetGamesOpts {
            page_number: 0,
            page_size: 20,
            //filter_language: Some("Latin".into()),
            //filter_genre: Some("Action".into()),
            search: Some("Mine".into()),
            ..Default::default()
        };
        let res = database.get_games(&opts).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_banner() -> Result<(), ChadError> {
        let database = DatabaseFetcher::new(None);
        let banner = database.find_banner("Minecraft").await;
        println!("{:#?}", banner);
        Ok(())
    }
}

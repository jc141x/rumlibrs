use std::collections::BTreeSet;

#[cfg(feature = "database")]
use crate::database;
use futures::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrapeError {
    #[error("HTTP Error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Url Parse Error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Failed to scrape game: {message} (url: {url})")]
    Game { message: String, url: reqwest::Url },

    #[error("Failed to scrape page {page}: {message}")]
    Page { message: String, page: usize },

    #[error("Failed to scrape: {0}")]
    Other(String),
}

impl ScrapeError {
    pub fn game(message: impl Into<String>, url: &reqwest::Url) -> Self {
        Self::Game {
            message: message.into(),
            url: url.clone(),
        }
    }

    pub fn page(message: impl Into<String>, page: usize) -> Self {
        Self::Page {
            message: message.into(),
            page,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    /// Infohash of the torrent
    pub hash: String,
    /// Name of the game
    pub name: String,
    /// Version of the game
    pub version: Option<String>,
    /// Description of the game
    pub description: String,
    /// Id from 1337x, used for sorting
    pub id: usize,
    /// Size
    pub size: String,
    /// List of genres
    pub genres: BTreeSet<String>,
    /// List of tags
    pub tags: BTreeSet<String>,
    /// List of languages
    pub languages: BTreeSet<String>,
}

impl Into<database::Game> for Game {
    fn into(self) -> database::Game {
        database::Game {
            game: database::table::Game {
                hash: self.hash,
                name: self.name,
                version: self.version,
                description: self.description,
                banner_rel_path: None,
                data_added: None,
                leetx_id: self.id,
            },
            genres: self.genres,
            tags: self.tags,
            languages: self.languages,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeetxScraper {
    base_url: String,
    uploader: String,
    page_buf_factor: usize,
    game_buf_factor: usize,
}

impl Default for LeetxScraper {
    fn default() -> Self {
        Self::new("https://1337x.to", "johncena141", 3, 10)
    }
}

impl LeetxScraper {
    pub fn new(
        base_url: impl Into<String>,
        uploader: impl Into<String>,
        page_buf_factor: usize,
        game_buf_factor: usize,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            uploader: uploader.into(),
            page_buf_factor,
            game_buf_factor,
        }
    }

    fn get_pages_try_futures(
        &self,
        first_page: usize,
    ) -> impl Stream<Item = impl TryFuture<Ok = Vec<String>, Error = ScrapeError>> {
        let (base_url, uploader) = (self.base_url.clone(), self.uploader.clone());
        stream::iter(first_page..)
            .map(move |i| Self::parse_page(i, base_url.clone(), uploader.clone()))
    }

    fn get_pages_futures(
        &self,
        first_page: usize,
    ) -> impl Stream<
        Item = impl Future<Output = Box<dyn Iterator<Item = Result<String, ScrapeError>> + Send>>,
    > + Send {
        self.get_pages_try_futures(first_page).map(|future| {
            future.map_ok_or_else(
                |err| {
                    Box::new(std::iter::once(Err(err)))
                        as Box<dyn Iterator<Item = Result<_, _>> + Send>
                },
                |page: Vec<String>| {
                    Box::new(page.into_iter().map(|url| Ok(url)))
                        as Box<dyn Iterator<Item = Result<_, _>> + Send>
                },
            )
        })
    }

    fn get_urls_n_pages_buffered(
        &self,
        first_page: usize,
        num_pages: usize,
    ) -> impl TryStream<Ok = String, Error = ScrapeError> {
        self.get_pages_futures(first_page)
            .take(num_pages)
            .buffered(self.page_buf_factor)
            .flat_map(|page| stream::iter(page))
    }

    pub fn get_games_n_pages(
        &self,
        first_page: usize,
        num_pages: usize,
    ) -> impl Stream<Item = Result<Game, ScrapeError>> {
        let base_url = self.base_url.clone();
        self.get_urls_n_pages_buffered(first_page, num_pages)
            .map_ok(move |url| Self::parse_game(url, base_url.clone()))
            .try_buffered(self.game_buf_factor)
    }

    pub async fn collect_games_n_pages(
        &self,
        first_page: usize,
        num_pages: usize,
    ) -> Vec<Result<Game, ScrapeError>> {
        self.get_games_n_pages(first_page, num_pages)
            .collect()
            .await
    }

    /// Gets a stream that will yield all games ordered from new to old.
    ///
    /// Pages and games will be scraped asynchronously to maximize speed.
    ///
    /// Useful for interactively sending events to a GUI whenever a game is scraped.
    ///
    /// Note: some results yielded by the stream may be errors. This can either be an error that
    /// occurred when scraping a single game or when scraping a page. This means that the list of
    /// returned games might be incomplete in case errors occur.
    ///
    /// ```rust
    /// let leetx_scraper = LeetxScraper::default();
    ///
    /// let games_stream = leetx_scraper.get_all_games().await.unwrap();
    ///
    /// tokio::pin!(games_stream);
    ///
    /// while let Some(item) = games_stream.next().await {
    ///     match item {
    ///         Ok(game) => println!("Successfully scraped {:#?}", game.name),
    ///         Err(err) => println!("An error occurred while scraping: {:#?}", err),
    ///     }
    /// }
    /// ```
    pub async fn get_all_games(
        &self,
    ) -> Result<impl Stream<Item = Result<Game, ScrapeError>>, ScrapeError> {
        let num_pages = self.get_num_pages().await?;
        Ok(self.get_games_n_pages(1, num_pages))
    }

    /// Collect all games into a Vec
    ///
    /// Benefits from the same performance characteristics as `get_all_games`, but games are only
    /// returned when all games are scraped.
    ///
    /// Note: some results in the returned vector may be errors. This can either be an error that
    /// occurred when scraping a single game or when scraping a page. This means that the list of
    /// returned games might be incomplete in case errors occur.
    ///
    /// ```rust
    /// let leetx_scraper = LeetxScraper::default();
    ///
    /// let games: Vec<Result<Game, ScrapeError>> = leetx_scraper.collect_all_games().await.unwrap();
    /// ```
    pub async fn collect_all_games(&self) -> Result<Vec<Result<Game, ScrapeError>>, ScrapeError> {
        Ok(self.get_all_games().await?.collect().await)
    }

    fn parse_tags(subtitle: &str) -> BTreeSet<String> {
        lazy_static! {
            static ref RE_TAGS: Regex = Regex::new(r"\[(.*?)\]").unwrap();
        }

        RE_TAGS
            .captures_iter(subtitle)
            .map(|c| c[1].to_string())
            .filter(|t| t != "johncena141")
            .map(|t| match t.as_str() {
                "GNU/Linux Wine" => "Wine".into(),
                "GNU/Linux Native" => "Native".into(),
                _ => t,
            })
            .collect()
    }

    fn parse_items(line: &str) -> Result<BTreeSet<String>, ScrapeError> {
        let mut list = line
            .split(":")
            .skip(1)
            .next()
            .ok_or(ScrapeError::Other("items list".into()))?;
        list = list.strip_prefix(" ").unwrap_or(list);
        list = list.strip_suffix("\n").unwrap_or(list);
        Ok(list
            .split(",")
            .map(|i| i.strip_prefix(" ").unwrap_or(i).to_string())
            .collect())
    }

    async fn parse_game(
        url: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Game, ScrapeError> {
        //let url = format!("{}{}", base_url, url);
        let url = reqwest::Url::parse(&base_url.into())?.join(&url.into())?;

        lazy_static! {
            static ref RE_ID: Regex = Regex::new(r".*/(\d*)/.*").unwrap();
        }

        let id = RE_ID
            .captures_iter(url.as_str())
            .next()
            .and_then(|c| c[1].parse::<usize>().ok())
            .ok_or(ScrapeError::game("1337x id from url", &url))?;

        let page_html = reqwest::get(url.clone()).await?.text().await?;

        // To prevent cloudflare from fucking my while testing
        //let page_html = include_str!("test_game.html");

        let document = Html::parse_document(&page_html);

        lazy_static! {
            static ref NAME_SELECTOR: Selector =
                Selector::parse("#description p.align-center strong").unwrap();
        }

        let name = document
            .select(&NAME_SELECTOR)
            .next()
            .and_then(|e| e.text().next())
            .map(|n| n.strip_suffix(" ").unwrap_or(n))
            .ok_or(ScrapeError::game("name", &url))?;

        lazy_static! {
            static ref SUBTITLE_SELECTOR: Selector =
                Selector::parse("#description p.align-center span").unwrap();
        }

        let subtitle = document
            .select(&SUBTITLE_SELECTOR)
            .skip(1)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ScrapeError::game("subtitle", &url))?;

        let version = if let Some(v) = subtitle.split(" ").next() {
            if v.contains("[") && v.contains("]") {
                None
            } else {
                Some(v)
            }
        } else {
            None
        };

        let tags = Self::parse_tags(&subtitle);

        lazy_static! {
            static ref HASH_SELECTOR: Selector = Selector::parse(".infohash-box span").unwrap();
        }

        let hash = document
            .select(&HASH_SELECTOR)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ScrapeError::game("hash", &url))?;

        lazy_static! {
            static ref SIZE_SELECTOR: Selector =
                Selector::parse(".box-info .list li span").unwrap();
        }

        let size = document
            .select(&SIZE_SELECTOR)
            .skip(3)
            .next()
            .and_then(|span| span.text().next())
            .ok_or(ScrapeError::game("size", &url))?;

        lazy_static! {
            static ref DESCRIPTION_SELECTOR: Selector = Selector::parse("#description p").unwrap();
            static ref STRONG_SELECTOR: Selector = Selector::parse("strong").unwrap();
        }

        let info_box = document
            .select(&DESCRIPTION_SELECTOR)
            .skip_while(|s| {
                s.select(&STRONG_SELECTOR).all(|s| {
                    s.text()
                        .next()
                        .map(|s| !s.contains("Info") && !s.contains("Description"))
                        .unwrap_or(false)
                })
            })
            .next()
            .ok_or(ScrapeError::game("info", &url))?;

        let mut description_state = false;
        let mut description = String::new();
        let mut genres: BTreeSet<String> = BTreeSet::new();
        let mut languages: BTreeSet<String> = BTreeSet::new();

        for text in info_box.text() {
            if description_state {
                description.push_str(&text.strip_prefix(" ").unwrap_or(&text));
            } else if text.contains("Genre:") {
                genres = Self::parse_items(text)?;
            } else if text.contains("Language:") {
                languages = Self::parse_items(text)?;
            } else if text.contains("Description") {
                description_state = true;
            }
        }

        Ok(Game {
            id,
            name: name.into(),
            hash: hash.into(),
            version: version.map(|v| v.into()),
            size: size.into(),
            tags,
            description,
            genres,
            languages,
        })
    }

    async fn get_num_pages(&self) -> Result<usize, ScrapeError> {
        let page_url = format!("{}/{}-torrents/1/", self.base_url, self.uploader);
        let page_html = reqwest::get(page_url).await?.text().await?;
        let document = Html::parse_document(&page_html);

        lazy_static! {
            static ref LINK_SELECTOR: Selector =
                Selector::parse("div.pagination li.last a").unwrap();
        }

        let href = document
            .select(&LINK_SELECTOR)
            .next()
            .map(|l| l.value().attr("href").map(|s| s.to_string()))
            .flatten()
            .ok_or(ScrapeError::page("pagination", 1))?;

        lazy_static! {
            static ref RE_PAGE: Regex = Regex::new(r".*/(\d*)/.*").unwrap();
        }

        let page = RE_PAGE
            .captures_iter(&href)
            .next()
            .and_then(|c| c[1].parse::<usize>().ok())
            .ok_or(ScrapeError::page("page number from pagination link", 1))?;

        Ok(page)
    }

    pub async fn parse_page(
        page: usize,
        base_url: impl std::fmt::Display,
        uploader: impl std::fmt::Display,
    ) -> Result<Vec<String>, ScrapeError> {
        let page_url = format!("{}/{}-torrents/{}/", base_url, uploader, page);

        let page_html = reqwest::get(page_url).await?.text().await?;

        // To prevent cloudflare from fucking my while testing
        //let page_html = include_str!("page1.html");

        let document = Html::parse_document(&page_html);

        lazy_static! {
            static ref LINK_SELECTOR: Selector =
                Selector::parse("td.coll-1 a:nth-child(2)").unwrap();
        }

        let links = document
            .select(&LINK_SELECTOR)
            .filter_map(|l| l.value().attr("href").map(|s| s.to_string()))
            .collect::<Vec<_>>();

        Ok(links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_page() {
        LeetxScraper::parse_page(1, "https://1337x.to", "johncena141")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_parse_game() {
        let game = LeetxScraper::parse_game(
            "/torrent/4973380/Invisible-Inc-b281021-ENG-GOG-GNU-Linux-Native-johncena141/",
            "https://1337x.to",
        )
        .await
        .unwrap();

        println!("{:#?}", game);
    }

    #[tokio::test]
    async fn test_num_pages() {
        let leetx_scraper = LeetxScraper::default();
        println!("{}", leetx_scraper.get_num_pages().await.unwrap());
    }

    /*
    #[tokio::test]
    async fn test_get_pages() {
        let leetx_scraper = LeetxScraper::default();

        let games_stream = leetx_scraper.get_all_games().await.unwrap();

        tokio::pin!(games_stream);

        while let Some(item) = games_stream.next().await {
            match item {
                Ok(game) => println!("{:#?}", game.name),
                Err(err) => println!("{:#?}", err),
            }
        }
    }*/
}

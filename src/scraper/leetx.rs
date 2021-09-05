use futures::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::util::ChadError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    /// Infohash of the torrent
    pub hash: String,
    /// Name of the game
    pub name: String,
    /// Version of the game
    pub version: String,
    /// Description of the game
    pub description: String,
    /// Id from 1337x, used for sorting
    pub id: usize,
    /// Size
    pub size: String,
    /// List of genres
    pub genres: Vec<String>,
    /// List of tags
    pub tags: Vec<String>,
    /// List of languages
    pub languages: Vec<String>,
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
    ) -> impl Stream<Item = impl TryFuture<Ok = Vec<String>, Error = ChadError>> {
        let (base_url, uploader) = (self.base_url.clone(), self.uploader.clone());
        stream::iter(1..).map(move |i| {
            Self::parse_page(i, base_url.clone(), uploader.clone()) //.unwrap_or_else(|_| Vec::new())
        })
    }

    fn get_pages_futures(
        &self,
    ) -> impl Stream<Item = impl Future<Output = Box<dyn Iterator<Item = Result<String, ChadError>>>>>
    {
        self.get_pages_try_futures().map(|future| {
            future.map_ok_or_else(
                |err| Box::new(std::iter::once(Err(err))) as Box<dyn Iterator<Item = Result<_, _>>>,
                |page: Vec<String>| {
                    Box::new(page.into_iter().map(|url| Ok(url)))
                        as Box<dyn Iterator<Item = Result<_, _>>>
                },
            )
        })
    }

    fn get_urls_n_pages_buffered(
        &self,
        num_pages: usize,
    ) -> impl TryStream<Ok = String, Error = ChadError> {
        self.get_pages_futures()
            .take(num_pages)
            .buffered(self.page_buf_factor)
            .flat_map(|page| stream::iter(page))
    }

    //fn get_games_n_pages_buffered(&self, n: usize) -> impl TryStream<Ok = Game, Error = ChadError> {
    fn get_games_n_pages_buffered(&self, n: usize) -> impl Stream<Item = Result<Game, ChadError>> {
        let base_url = self.base_url.clone();
        self.get_urls_n_pages_buffered(n)
            .map_ok(move |url| Self::parse_game(url, base_url.clone()))
            .try_buffered(self.game_buf_factor)
    }

    fn parse_tags(subtitle: &str) -> Vec<String> {
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

    fn parse_items(line: &str) -> Result<Vec<String>, ChadError> {
        let mut list = line
            .split(":")
            .skip(1)
            .next()
            .ok_or(ChadError::scrape_error("Failed to parse items list"))?;
        list = list.strip_prefix(" ").unwrap_or(list);
        list = list.strip_suffix("\n").unwrap_or(list);
        Ok(list
            .split(",")
            .map(|i| i.strip_prefix(" ").unwrap_or(i).to_string())
            .collect())
    }

    pub async fn parse_game(
        url: impl std::fmt::Display,
        base_url: impl std::fmt::Display,
    ) -> Result<Game, ChadError> {
        let url = format!("{}{}", base_url, url);

        lazy_static! {
            static ref RE_ID: Regex = Regex::new(r".*/(\d*)/.*").unwrap();
        }

        let id = RE_ID
            .captures_iter(&url)
            .next()
            .and_then(|c| c[1].parse::<usize>().ok())
            .ok_or(ChadError::scrape_error(format!(
                "Failed to scrape 1337x id from url ({})",
                &url
            )))?;

        let page_html = reqwest::get(&url).await?.text().await?;

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
            .ok_or(ChadError::scrape_error(format!(
                "Failed to scrape name ({})",
                &url
            )))?;

        lazy_static! {
            static ref SUBTITLE_SELECTOR: Selector =
                Selector::parse("#description p.align-center span").unwrap();
        }

        let subtitle = document
            .select(&SUBTITLE_SELECTOR)
            .skip(1)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ChadError::scrape_error(format!(
                "Failed to scrape subtitle ({})",
                &url
            )))?;

        let version = subtitle.split(" ").next().unwrap_or("unknown");

        let tags = Self::parse_tags(&subtitle);

        lazy_static! {
            static ref HASH_SELECTOR: Selector = Selector::parse(".infohash-box span").unwrap();
        }

        let hash = document
            .select(&HASH_SELECTOR)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ChadError::scrape_error(format!(
                "Failed to scrape hash ({})",
                &url
            )))?;

        lazy_static! {
            static ref SIZE_SELECTOR: Selector =
                Selector::parse(".box-info .list li span").unwrap();
        }

        let size = document
            .select(&SIZE_SELECTOR)
            .skip(3)
            .next()
            .and_then(|span| span.text().next())
            .ok_or(ChadError::scrape_error(format!(
                "Failed to scrape size ({})",
                &url
            )))?;

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
            .ok_or(ChadError::scrape_error(format!(
                "Failed to game info ({})",
                &url
            )))?;

        let mut description_state = false;
        let mut description = String::new();
        let mut genres: Vec<String> = Vec::new();
        let mut languages: Vec<String> = Vec::new();

        for text in info_box.text() {
            if description_state {
                description.push_str(&text);
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
            version: version.into(),
            size: size.into(),
            tags,
            description,
            genres,
            languages,
        })
    }

    pub async fn parse_page(
        page: usize,
        base_url: impl std::fmt::Display,
        uploader: impl std::fmt::Display,
    ) -> Result<Vec<String>, ChadError> {
        let page_url = format!("{}/{}-torrents/{}/", base_url, uploader, page);
        println!("{}", page_url);

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
    async fn test_get_pages() {
        let leetx_scraper = LeetxScraper::default();

        let games_stream = leetx_scraper.get_games_n_pages_buffered(26);

        tokio::pin!(games_stream);

        while let Some(item) = games_stream.next().await {
            match item {
                Ok(game) => println!("{:#?}", game.name),
                Err(err) => println!("{:#?}", err),
            }
        }
    }
}

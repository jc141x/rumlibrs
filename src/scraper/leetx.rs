use async_stream::try_stream;
use futures::future::try_join_all;

use futures::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
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
    buf_factor: usize,
}

impl LeetxScraper {
    pub fn new(base_url: impl Into<String>, uploader: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            uploader: uploader.into(),
            buf_factor: 5,
        }
    }

    /*
    pub fn scrape(&self) -> impl Stream<Item = Result<Game, ChadError>> {
        self.get_pages_futures()
            .flat_map(|url| Self::parse_game(url, base_url))
    }
    */

    fn get_pages_futures(
        &self,
    ) -> impl Stream<Item = impl TryFuture<Ok = Vec<String>, Error = ChadError>> {
        let (base_url, uploader) = (self.base_url.clone(), self.uploader.clone());
        stream::iter(0..).map(move |i| Self::parse_page(i, base_url.clone(), uploader.clone()))
    }

    fn get_urls_buffered(
        &self,
        num_pages: usize,
    ) -> impl TryStream<Item = String, Error = ChadError> {
        self.get_pages_futures()
            .filter_map(|res| res.ok())
            .take(num_pages)
            .buffered(self.buf_factor)
            .flat_map(|page| stream::iter(page))
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
        url: &str,
        base_url: impl std::fmt::Display,
    ) -> Result<Game, ChadError> {
        let url = format!("{}/{}", base_url, url);
        //println!("Url: {}", url);

        lazy_static! {
            static ref RE_ID: Regex = Regex::new(r".*/(\d*)/.*").unwrap();
        }

        let id = RE_ID
            .captures_iter(&url)
            .next()
            .and_then(|c| c[1].parse::<usize>().ok())
            .ok_or(ChadError::scrape_error(
                "Failed to scrape 1337x id from url",
            ))?;

        //println!("Id: {}", id);

        //let page_html = reqwest::get(url).await?.text().await?;

        // To prevent cloudflare from fucking my while testing
        let page_html = include_str!("test_game.html");

        let document = Html::parse_document(&page_html);

        lazy_static! {
            static ref NAME_SELECTOR: Selector =
                Selector::parse("#description p.align-center strong").unwrap();
        }

        let name = document
            .select(&NAME_SELECTOR)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ChadError::scrape_error("Failed to scrape name"))?;

        //println!("Name: {}", name);

        lazy_static! {
            static ref SUBTITLE_SELECTOR: Selector =
                Selector::parse("#description p.align-center span").unwrap();
        }

        let subtitle = document
            .select(&SUBTITLE_SELECTOR)
            .skip(1)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ChadError::scrape_error("Failed to scrape subtitle"))?;

        let version = subtitle.split(" ").next().unwrap_or("unknown");

        //println!("Version: {}", version);

        let tags = Self::parse_tags(&subtitle);

        //println!("Tags: {:#?}", tags);

        lazy_static! {
            static ref HASH_SELECTOR: Selector = Selector::parse(".infohash-box span").unwrap();
        }

        let hash = document
            .select(&HASH_SELECTOR)
            .next()
            .and_then(|e| e.text().next())
            .ok_or(ChadError::scrape_error("Failed to scrape hash"))?;

        //println!("Hash: {}", hash);

        lazy_static! {
            static ref SIZE_SELECTOR: Selector =
                Selector::parse(".box-info .list li span").unwrap();
        }

        let size = document
            .select(&SIZE_SELECTOR)
            .skip(3)
            .next()
            .and_then(|span| span.text().next())
            .ok_or(ChadError::scrape_error("Failed to scrape size"))?;

        //println!("Size: {}", size);

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
                        .map(|s| !s.contains("Info"))
                        .unwrap_or(false)
                })
            })
            .next()
            .ok_or(ChadError::scrape_error("Failed to game info"))?;

        //println!("{:#?}", info_box.text().collect::<Vec<_>>());

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

        //println!("Description: {}", description);

        //println!("Genres: {:#?}", genres);
        //println!("Languages: {:#?}", languages);

        Ok(Game {
            id,
            name: name.into(),
            hash: hash.into(),
            version: version.into(),
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

        //let page_html = reqwest::get(page_url).await?.text().await?;

        // To prevent cloudflare from fucking my while testing
        let page_html = include_str!("page1.html");

        let document = Html::parse_document(&page_html);

        lazy_static! {
            static ref LINK_SELECTOR: Selector =
                Selector::parse("td.coll-1 a:nth-child(2)").unwrap();
        }

        let links = document
            .select(&LINK_SELECTOR)
            .filter_map(|l| l.value().attr("href").map(|s| s.to_string()))
            .collect::<Vec<_>>();

        println!("{:#?}", links);

        Ok(links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_page() {
        let leetx_scraper = LeetxScraper::new("https://1337x.to", "johncena141");
        LeetxScraper::parse_page(1, "https://1337x.to", "johncena141")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_parse_game() {
        let leetx_scraper = LeetxScraper::new("https://1337x.to", "johncena141");
        let game = LeetxScraper::parse_game(
            "/torrent/4973380/Invisible-Inc-b281021-ENG-GOG-GNU-Linux-Native-johncena141/",
            "https://1337x.to",
        )
        .await
        .unwrap();

        println!("{:#?}", game);
    }
}

use futures::prelude::*;
use steamgriddb_api::{
    query_parameters::{GridDimentions, GridQueryParameters},
    Client, QueryType,
};

use crate::util::ChadError;

pub struct BannerFetcher {
    key: String,
}

impl BannerFetcher {
    pub fn new(steamgriddb_key: &str) -> Self {
        Self {
            key: steamgriddb_key.into(),
        }
    }

    async fn get_images(
        game_id: usize,
        key: impl Into<String>,
    ) -> Result<Vec<steamgriddb_api::images::Image>, ChadError> {
        let mut parameters = GridQueryParameters::default();
        parameters.dimentions = Some(&[GridDimentions::D460x215, GridDimentions::D920x430]);
        let query = QueryType::Grid(Some(parameters));
        Client::new(key)
            .get_images_for_id(game_id, &query)
            .map_err(|_| ChadError::message("Failed to find images"))
            .await
    }

    fn get_game_try_futures(
        &self,
        games: Vec<steamgriddb_api::search::SearchResult>,
    ) -> impl Stream<Item = impl TryFuture<Ok = Vec<steamgriddb_api::images::Image>, Error = ChadError>>
    {
        let key = self.key.clone();
        stream::iter(games).map(move |game| Self::get_images(game.id, key.clone()))
    }

    fn get_game_futures(
        &self,
        games: Vec<steamgriddb_api::search::SearchResult>,
    ) -> impl Stream<
        Item = impl Future<
            Output = Box<
                dyn Iterator<Item = Result<steamgriddb_api::images::Image, ChadError>> + Send,
            >,
        >,
    > {
        self.get_game_try_futures(games).map(|future| {
            future.map_ok_or_else(
                |err| {
                    Box::new(std::iter::once(Err(err)))
                        as Box<
                            dyn Iterator<Item = Result<steamgriddb_api::images::Image, ChadError>>
                                + Send,
                        >
                },
                |page: Vec<steamgriddb_api::images::Image>| {
                    Box::new(page.into_iter().map(|url| Ok(url)))
                        as Box<
                            dyn Iterator<Item = Result<steamgriddb_api::images::Image, ChadError>>
                                + Send,
                        >
                },
            )
        })
    }

    pub async fn find_images(&self, game_name: &str) -> Result<Vec<String>, ChadError> {
        let games = Client::new(&self.key)
            .search(game_name)
            .await
            .map_err(|_| ChadError::message("Failed to search game"))?;

        let results: Vec<_> = self
            .get_game_futures(games)
            .buffered(5)
            .flat_map(|game| stream::iter(game))
            .collect()
            .await;

        Ok(results
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|g| g.url)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_images() {
        if let Ok(key) = std::env::var("STEAMGRIDDB_KEY") {
            let fetcher = BannerFetcher::new(&key);
            let images = fetcher.find_images("Minecraft").await.unwrap();
            println!("{:#?}", images);
        }
    }
}

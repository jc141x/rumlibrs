use futures::prelude::*;
use steamgriddb_api::{
    query_parameters::{GridDimentions, GridQueryParameters},
    Client, QueryType,
};

use crate::util::ChadError;

pub struct BannerFetcher {
    client: Client,
}

impl BannerFetcher {
    pub fn new(steamgriddb_key: &str) -> Self {
        Self {
            client: Client::new(steamgriddb_key),
        }
    }

    async fn get_images(
        &self,
        game_id: usize,
    ) -> Result<Vec<steamgriddb_api::images::Image>, ChadError> {
        let mut parameters = GridQueryParameters::default();
        parameters.dimentions = Some(&[GridDimentions::D460x215, GridDimentions::D920x430]);
        let query = QueryType::Grid(Some(parameters));
        self.client
            .get_images_for_id(game_id, &query)
            .map_err(|_| ChadError::message("Failed to find images"))
            .await
    }

    pub async fn find_images(&self, game_name: &str) -> Result<Vec<String>, ChadError> {
        let games = self
            .client
            .search(game_name)
            .await
            .map_err(|_| ChadError::message("Failed to search game"))?;

        let results: Vec<_> = stream::iter(games)
            .map(|game| self.get_images(game.id))
            .map(|future| {
                future.map_ok_or_else(
                    |err| {
                        Box::new(std::iter::once(Err(err)))
                            as Box<dyn Iterator<Item = Result<_, _>> + Send>
                    },
                    |page: Vec<steamgriddb_api::images::Image>| {
                        Box::new(page.into_iter().map(|url| Ok(url)))
                            as Box<
                                dyn Iterator<Item = Result<steamgriddb_api::images::Image, _>>
                                    + Send,
                            >
                    },
                )
            })
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

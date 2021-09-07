use futures::prelude::*;
use magick_rust::{magick_wand_genesis, MagickWand};
use std::sync::Once;
use steamgriddb_api::{
    query_parameters::{GridDimentions, GridQueryParameters},
    Client, QueryType,
};

use crate::util::ChadError;

static START: Once = Once::new();

pub fn scale_compress_image(image: impl AsRef<[u8]>) -> Result<Vec<u8>, ChadError> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let wand = MagickWand::new();
    wand.read_image_blob(image)?;
    wand.sharpen_image(0., 1.)?;
    wand.resize_image(460, 215, magick_rust::bindings::FilterType_QuadraticFilter);

    let image = wand.write_image_blob("png")?;
    Ok(image)
}

async fn get_images(
    client: &Client,
    game_id: usize,
) -> Result<Vec<steamgriddb_api::images::Image>, ChadError> {
    let mut parameters = GridQueryParameters::default();
    parameters.dimentions = Some(&[GridDimentions::D460x215, GridDimentions::D920x430]);
    let query = QueryType::Grid(Some(parameters));
    client
        .get_images_for_id(game_id, &query)
        .map_err(|_| ChadError::message("Failed to find images"))
        .await
}

pub async fn find_images(steamgriddb_key: &str, game_name: &str) -> Result<Vec<String>, ChadError> {
    let client = Client::new(steamgriddb_key);
    let games = client
        .search(game_name)
        .await
        .map_err(|_| ChadError::message("Failed to search game"))?;

    let results: Vec<_> = stream::iter(games)
        .map(|game| get_images(&client, game.id))
        .map(|future| {
            future.map_ok_or_else(
                |err| Box::new(std::iter::once(Err(err))) as Box<dyn Iterator<Item = Result<_, _>>>,
                |page: Vec<steamgriddb_api::images::Image>| {
                    Box::new(page.into_iter().map(|url| Ok(url)))
                        as Box<dyn Iterator<Item = Result<steamgriddb_api::images::Image, _>>>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scale_compress() {
        let banner = std::fs::read("banner.png").unwrap();
        std::fs::write(
            "test_scale_banner_out.png",
            scale_compress_image(banner).unwrap(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_find_images() {
        if let Ok(key) = std::env::var("STEAMGRIDDB_KEY") {
            let images = find_images(&key, "Minecraft").await.unwrap();
            println!("{:#?}", images);
        }
    }
}

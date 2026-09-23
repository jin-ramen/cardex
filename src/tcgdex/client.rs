use image::{RgbImage, RgbaImage};
use serde::de::DeserializeOwned;

use crate::tcgdex::SetBrief;

use super::query::Query;
use super::error::{TcgDexError, ProblemDetails};
use super::card::{Card, CardBrief};
use super::set::Set;

pub struct TcgDexClient { client: reqwest::Client, base_url: String }

impl TcgDexClient {
    pub fn new(region: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url:  format!("https://api.tcgdex.net/v2/{region}"),
        }
    }

    async fn get<T: DeserializeOwned>(
        &self, 
        path: &str,
        queries: &[Query],
    ) -> Result<T, TcgDexError> {
        let url = format!("{}{path}", self.base_url);
        let pairs: Vec<(&str, &str)> = queries.iter().map(Query::as_pair).collect();
        
        let resp = self.client.get(&url).query(&pairs).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            if let Ok(problem) = serde_json::from_str::<ProblemDetails>(&body) {
                return Err(TcgDexError::Api(problem));
            }
            return Err(TcgDexError::Status { status, url });
        }

        Ok(serde_json::from_str(&body)?)
    }


    pub async fn get_card(&self, id: &str) -> Result<Card, TcgDexError> {
        self.get(&format!("/cards/{id}"), &[]).await
    }

    pub async fn list_cards(&self, queries: &[Query]) -> Result<Vec<CardBrief>, TcgDexError> {
        self.get(&format!("/cards/"), queries).await
    }

    pub async fn get_set(&self, id: &str) -> Result<Set, TcgDexError> {
        self.get(&format!("/sets/{id}"), &[]).await
    }

    pub async fn list_sets(&self, queries: &[Query]) -> Result<Vec<SetBrief>, TcgDexError> {
        self.get("/sets", queries).await
    }

    pub async fn list_rarities(&self) -> Result<Vec<String>, TcgDexError> {
        self.get("/rarities", &[]).await
    }

    pub async fn fetch_image(&self, url: &str) -> Result<Vec<u8>, TcgDexError> {
        let bytes = self.client.get(url).send().await?.error_for_status()?.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn card_art(&self, card: &Card) -> Result<Option<RgbImage>, TcgDexError> {
        match card.image_url() {
            Some(url) => Ok(Some(image::load_from_memory(&self.fetch_image(&url).await?)?.to_rgb8())),
            None => Ok(None),
        }
    }

    pub async fn set_logo(&self, set: &Set) -> Result<Option<RgbaImage>, TcgDexError> {
        match &set.logo {
            Some(base) => Ok(Some(self.fetch_sprite(&format!("{base}.png")).await?)),
            None => Ok(None),
        }
    }

    pub async fn set_symbol(&self, set: &Set) -> Result<Option<RgbaImage>, TcgDexError> {
        match &set.symbol {
            Some(base) => Ok(Some(self.fetch_sprite(&format!("{base}.png")).await?)),
            None => Ok(None),
        }
    }
 
    async fn fetch_sprite(&self, url: &str) -> Result<RgbaImage, TcgDexError> {
        Ok(image::load_from_memory(&self.fetch_image(url).await?)?.to_rgba8())
    }
}
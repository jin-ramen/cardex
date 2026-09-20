use serde::de::DeserializeOwned;

use super::error::{TcgDexError, ProblemDetails};
use super::models::{Card, Set};

pub struct TcgDexClient { client: reqwest::Client, base_url: String }

impl TcgDexClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url:  "https://api.tcgdex.net/v2/en".to_string(),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, TcgDexError> {
        let url = format!("{}{path}", self.base_url);
        let resp = reqwest::get(&url).await?;
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
        self.get(&format!("/cards/{id}")).await

    }

    pub async fn get_set(&self, id: &str) -> Result<Set, TcgDexError> {
        self.get(&format!("/sets/{id}")).await
    }

    pub async fn list_rarities(&self) -> Result<Vec<String>, TcgDexError> {
        self.get("/rarities").await
    }

    pub async fn list_sets(&self) -> Result<Vec<Set>, TcgDexError> {
        self.get("/sets").await
    }
}
use std::fmt;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ProblemType {
    #[serde(rename = "https://tcgdex.dev/errors/general")]
    General,
    #[serde(rename = "https://tcgdex.dev/errors/language-invalid")]
    LanguageInvalid,
    #[serde(rename = "https://tcgdex.dev/errors/not-found")]
    NotFound,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, thiserror::Error)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub kind: ProblemType,
    pub title: String,
    pub status: u16,
    pub endpoint: String,
    pub method: String,
    pub lang: Option<String>,
    pub details: Option<String>,
}

impl fmt::Display for ProblemDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TCGdex {} {} -> {}: ", self.method, self.endpoint, self.status)?;
        let msg = self.details.as_deref().unwrap_or(&self.title);
        match (self.kind, &self.lang) {
            (ProblemType::LanguageInvalid, Some(lang)) => write!(f, "{msg} (lang: {lang})"),
            (ProblemType::General, _) => write!(f, "server error: {msg}"),
            (ProblemType::Unknown, _) => write!(f, "unclassified error: {msg}"),
            _ => f.write_str(msg),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TcgDexError {
    #[error("request to TCGdex failed")]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Api(#[from] ProblemDetails),

    #[error("TCGdex returned {status} for {url} with an unrecognised body")]
    Status { status: reqwest::StatusCode, url: String },

    #[error("could not decode TCGdex response")]
    Decode(#[from] serde_json::Error),

    #[error("could not decode image: {0}")]
    Image(#[from] image::ImageError),
}
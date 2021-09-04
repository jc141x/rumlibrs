use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChadError {
    #[error("Json (de)serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP Error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Database Error: {0}")]
    DatabaseError(DatabaseError),

    #[error("Scrape Error: {0}")]
    ScrapeError(String),

    #[error("Message: {0}")]
    Message(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ChadError {
    pub fn message<T: Into<String>>(message: T) -> Self {
        Self::Message(message.into())
    }

    pub fn scrape_error<T: Into<String>>(message: T) -> Self {
        Self::ScrapeError(message.into())
    }
}

#[derive(Debug, Error)]
pub struct DatabaseError(u16);

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.0 {
            503 => "pg connection err or insufficient resources",
            403 => "invalid grantor, role or auth specification",
            409 => "foreign key or uniqueness violation",
            405 => "read only sql transaction",
            500 => "general error",
            413 => "too complex",
            400 => "default code for “raise”",
            404 => "undefined function or table",
            401 => "insufficient privileges",
            100..=199 => "success", // Error: success :)
            _ => "unknown",
        };
        f.write_fmt(format_args!("{} {}", self.0, message))
    }
}

impl From<u16> for DatabaseError {
    fn from(status: u16) -> Self {
        Self(status)
    }
}

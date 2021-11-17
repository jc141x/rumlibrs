use thiserror::Error;

#[derive(Debug, Error)]
pub enum RumError {
    #[error("Json (de)serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Message: {0}")]
    Message(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl RumError {
    pub fn message<T: Into<String>>(message: T) -> Self {
        Self::Message(message.into())
    }
}

impl From<&str> for RumError {
    fn from(message: &str) -> Self {
        Self::message(message)
    }
}

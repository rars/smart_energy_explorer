use n3rgy_consumer_api_client::N3rgyClientError;

use crate::data::RepositoryError;

pub mod app;
pub mod electricity;
pub mod gas;
pub mod glowmarkt;
pub mod n3rgy;
pub mod profiles;
pub mod tariff;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Error: {0}")]
    Custom(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Date parse error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Failed request to n3rgy API: {0}")]
    N3rgyClientError(#[from] N3rgyClientError),
    #[error("Mutex '{name}' is poisoned")]
    MutexPoisonedError { name: String },
}

impl serde::Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

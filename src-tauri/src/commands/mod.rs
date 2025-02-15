use crate::{clients::glowmarkt::GlowmarktDataProviderError, data::RepositoryError};

pub mod app;
pub mod electricity;
pub mod gas;
pub mod glowmarkt;
pub mod profiles;
pub mod tariff;

pub(crate) const APP_SERVICE_NAME: &str = "io.github.rars.smart_energy_explorer";

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
    #[error("Failed interaction with Glowmarkt API: {0}")]
    GlowmarktApiError(#[from] GlowmarktDataProviderError),
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

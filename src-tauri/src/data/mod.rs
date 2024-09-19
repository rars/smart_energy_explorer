pub mod consumption;
pub mod energy_profile;
pub mod tariff;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Mutex guarding SQLite connection is poisoned")]
    SqliteConnectionMutexPoisonedError(),
}

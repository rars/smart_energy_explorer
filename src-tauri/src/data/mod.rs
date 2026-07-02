pub mod consumption;
pub mod energy_profile;
pub mod tariff;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Error with connection pool: {0}")]
    ConnectionPoolError(#[from] diesel::r2d2::PoolError),
}

use thiserror::Error;

/// App wide errors
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
}

use thiserror::Error;
use tracing_core::dispatcher::SetGlobalDefaultError;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Tracing error: {0}")]
    SetGlobalDefaultError(#[from] SetGlobalDefaultError),

    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error: {0}")]
    GenericError(#[from] anyhow::Error),
}

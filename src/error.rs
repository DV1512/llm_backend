use thiserror::Error;
use tracing_core::dispatcher::SetGlobalDefaultError;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ServerError {
    #[error("Tracing error: {0}")]
    SetGlobalDefault(#[from] SetGlobalDefaultError),
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error: {0}")]
    Generic(#[from] anyhow::Error),
}

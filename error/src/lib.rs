use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CraterError {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Network error: {0}")]
    Network(#[from] rtnetlink::Error),

    #[error("Serde json error: {0}")]
    Serde(#[from] serde_json::Error),
}

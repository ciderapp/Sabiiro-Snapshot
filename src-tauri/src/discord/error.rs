use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Error)]
pub enum DiscordError {
    #[error("Initialisation Error: {0}")]
    Init(String),
    #[error("No Client")]
    NoClient,
    #[error("Failed to Update Status: {0}")]
    Status(String)
}

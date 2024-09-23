use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Error)]
pub enum AirPlayError {
    #[error("Initialisation Error: {0}")]
    Init(String),
}

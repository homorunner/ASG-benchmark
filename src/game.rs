use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid game definition: {0}")]
    InvalidDefinition(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub game_type: String,
    pub rules: String,
    pub board_representation: String,
    pub move_representation: String,
}

impl Game {
    pub fn from_json(json_str: &str) -> Result<Self, GameError> {
        serde_json::from_str(json_str).map_err(|e| GameError::InvalidDefinition(e.to_string()))
    }

    pub fn to_json(&self) -> Result<String, GameError> {
        serde_json::to_string_pretty(self).map_err(|e| GameError::InvalidDefinition(e.to_string()))
    }
}

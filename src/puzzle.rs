use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PuzzleError {
    #[error("Invalid puzzle definition: {0}")]
    InvalidDefinition(String),
    #[error("File error: {0}")]
    FileError(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleScore {
    pub puzzle_id: String,
    pub score: f64,
    pub max_possible_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub id: String,
    pub description: String,
    pub game_states: Vec<String>,
    pub solutions: Vec<String>,
}

impl Puzzle {
    pub fn validate_solution(&self, results: &[String]) -> PuzzleScore {
        let mut score = 0.0;
        let n = self.game_states.len();

        for (i, result) in results.iter().enumerate() {
            if i < n && result == &self.solutions[i] {
                score += 1.0;
            }
        }

        PuzzleScore {
            puzzle_id: self.id.clone(),
            score,
            max_possible_score: n as f64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleCollection {
    pub name: String,
    pub description: String,
    pub game_type: String,
    pub goal: String,
    pub game_rule: String,
    pub puzzles: Vec<Puzzle>,
}

impl PuzzleCollection {
    pub fn load_from_file(file_path: &str) -> Result<Self, PuzzleError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| PuzzleError::FileError(e.to_string()))?;

        serde_json::from_str(&content).map_err(|e| PuzzleError::InvalidDefinition(e.to_string()))
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), PuzzleError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PuzzleError::InvalidDefinition(e.to_string()))?;

        std::fs::write(file_path, json).map_err(|e| PuzzleError::FileError(e.to_string()))
    }

    pub fn filter_by_game_type(&self, game_type: &str) -> Vec<&Puzzle> {
        if self.game_type == game_type {
            self.puzzles.iter().collect()
        } else {
            Vec::new()
        }
    }
}

//! BoardgameBench - A benchmark for evaluating LLM performance on abstract board game puzzles

pub mod evaluation;
pub mod game;
pub mod puzzle;

// Re-export commonly used types
pub use evaluation::{BenchmarkResult, BenchmarkRunner, Solver};
pub use game::{Game, GameError};
pub use puzzle::{Puzzle, PuzzleCollection, PuzzleError, PuzzleGoal, PuzzleScore};

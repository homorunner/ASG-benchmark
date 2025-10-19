use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;

use crate::puzzle::{Puzzle, PuzzleCollection, PuzzleScore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_name: String,
    pub solver_name: String,
    pub solver_description: String,
    pub total_puzzles: usize,
    pub total_score: f64,
    pub max_possible_score: f64,
    pub average_score: f64,
    pub puzzle_scores: Vec<PuzzleScore>,
    pub game_type_breakdown: Vec<GameTypeScore>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTypeScore {
    pub game_type: String,
    pub count: usize,
    pub average_score: f64,
}

pub struct BenchmarkRunner {
    pub puzzles: PuzzleCollection,
}

impl BenchmarkRunner {
    pub fn new(puzzles: PuzzleCollection) -> Self {
        Self { puzzles }
    }

    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let puzzles = PuzzleCollection::load_from_file(file_path)?;
        Ok(Self::new(puzzles))
    }

    pub fn run_benchmark(&self, solver: &Solver) -> BenchmarkResult {
        let puzzle_scores: Vec<PuzzleScore> = self
            .puzzles
            .puzzles
            .iter()
            .map(|puzzle| {
                let solution = solver.solve_puzzle(puzzle, &self.puzzles);
                puzzle.validate_solution(&solution)
            })
            .collect();

        let total_score: f64 = puzzle_scores.iter().map(|s| s.score).sum();
        let max_possible_score: f64 = puzzle_scores.iter().map(|s| s.max_possible_score).sum();
        let total_puzzles = puzzle_scores.len();
        let average_score = if max_possible_score > 0.0 {
            total_score / max_possible_score
        } else {
            0.0
        };

        // Calculate game type breakdown
        let mut game_type_scores: std::collections::HashMap<String, (usize, f64, f64)> =
            std::collections::HashMap::new();
        for score in &puzzle_scores {
            let entry = game_type_scores
                .entry(self.puzzles.game_type.clone())
                .or_insert((0, 0.0, 0.0));
            entry.0 += 1;
            entry.1 += score.score;
            entry.2 += score.max_possible_score;
        }

        let game_type_breakdown: Vec<GameTypeScore> = game_type_scores
            .into_iter()
            .map(|(game_type, (count, score, total_score))| GameTypeScore {
                game_type,
                count,
                average_score: if total_score > 0.0 {
                    score / total_score as f64
                } else {
                    0.0
                },
            })
            .collect();

        BenchmarkResult {
            benchmark_name: format!("{} on {}", solver.name(), self.puzzles.name),
            solver_name: solver.name().to_string(),
            solver_description: solver.description().to_string(),
            total_puzzles,
            total_score,
            max_possible_score,
            average_score,
            puzzle_scores,
            game_type_breakdown,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn run_benchmark_parallel(&self, solver: &Solver, num_threads: usize) -> BenchmarkResult {
        // Set the thread pool size for rayon
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .expect("Failed to build thread pool");

        let puzzle_scores: Vec<PuzzleScore> = self
            .puzzles
            .puzzles
            .par_iter()
            .map(|puzzle| {
                let solution = solver.solve_puzzle(puzzle, &self.puzzles);
                puzzle.validate_solution(&solution)
            })
            .collect();

        let total_score: f64 = puzzle_scores.iter().map(|s| s.score).sum();
        let max_possible_score: f64 = puzzle_scores.iter().map(|s| s.max_possible_score).sum();
        let total_puzzles = puzzle_scores.len();
        let average_score = if max_possible_score > 0.0 {
            total_score / max_possible_score
        } else {
            0.0
        };

        // Calculate game type breakdown
        let mut game_type_scores: std::collections::HashMap<String, (usize, f64, f64)> =
            std::collections::HashMap::new();
        for score in &puzzle_scores {
            let entry = game_type_scores
                .entry(self.puzzles.game_type.clone())
                .or_insert((0, 0.0, 0.0));
            entry.0 += 1;
            entry.1 += score.score;
            entry.2 += score.max_possible_score;
        }

        let game_type_breakdown: Vec<GameTypeScore> = game_type_scores
            .into_iter()
            .map(|(game_type, (count, score, total_score))| GameTypeScore {
                game_type,
                count,
                average_score: if total_score > 0.0 {
                    score / total_score as f64
                } else {
                    0.0
                },
            })
            .collect();

        BenchmarkResult {
            benchmark_name: format!("{} on {} (parallel)", solver.name(), self.puzzles.name),
            solver_name: solver.name().to_string(),
            solver_description: solver.description().to_string(),
            total_puzzles,
            total_score,
            max_possible_score,
            average_score,
            puzzle_scores,
            game_type_breakdown,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn export_results(
        &self,
        results: &BenchmarkResult,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(results)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn run_comparison(&self, solvers: &[&Solver]) -> Vec<BenchmarkResult> {
        solvers
            .iter()
            .map(|solver| self.run_benchmark(*solver))
            .collect()
    }
}

pub struct Solver {
    pub name: String,
    pub description: String,
    pub model: String,
    pub client: openai_api_rs::v1::api::Client,
}

impl Solver {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn test_api_reachability(&self) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = "Please respond with the single word 'hello' to me.";

        match tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(async { self.call_openai_api(prompt).await })
        {
            Ok(response) => Ok(response),
            Err(e) => Err(e),
        }
    }

    pub fn solve_puzzle(&self, puzzle: &Puzzle, puzzle_collection: &PuzzleCollection) -> Vec<String> {
        let mut results = Vec::new();
        let regex = Regex::new(r"\*\*Answer:\s*(\S+?)\*\*").unwrap();

        for i in 0..puzzle.game_states.len() {
            let prompt = self.build_prompt(puzzle, puzzle_collection, i);

            match tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime")
                .block_on(async { self.call_openai_api(&prompt).await })
            {
                Ok(response) => {
                    println!("Puzzle {} state {}\nResponse: {}", puzzle.id, i, response);

                    if let Some(caps) = regex.captures_iter(&response).last() {
                        let answer = caps
                            .get(1)
                            .map(|m| m.as_str().trim().to_lowercase())
                            .unwrap();
                        println!("Got {}, expected {}", answer, puzzle.solutions[i]);
                        results.push(answer);
                    } else {
                        eprintln!(
                            "No answer found in response for puzzle {} state {}",
                            puzzle.id, i
                        );
                        results.push("".to_string());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error calling OpenAI API for puzzle {} state {}: {}",
                        puzzle.id, i, e
                    );
                    results.push("".to_string());
                }
            }
        }

        results
    }
}

impl Solver {
    pub fn new(model: String) -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let base_url = env::var("OPENAI_BASE_URL").unwrap();

        let client = openai_api_rs::v1::api::Client::new_with_endpoint(base_url, api_key);

        Ok(Self {
            name: format!("OpenAI Solver ({})", model),
            description: format!("OpenAI API solver using {} model", model),
            model,
            client,
        })
    }

    fn build_prompt(&self, puzzle: &Puzzle, puzzle_collection: &PuzzleCollection, index: usize) -> String {
        let game_type = &puzzle_collection.game_type;
        let goal = &puzzle_collection.goal;
        let fen = &puzzle.game_states[index];
        format!(
            "You are a highly advanced AI specialized in solving abstract board game puzzles.
Your task is to analyze the given game state and provide a detailed strategic evaluation along with the best possible move.
Follow these guidelines to ensure optimal performance:
1. **Understanding the Game Rules**: Begin by thoroughly explaining the rules of {game_type} in the context of the current puzzle. Highlight unique aspects like movement patterns of pieces, special moves, and endgame conditions.
2. **Game State Analysis**: Assess the current state of the {game_type} board. Identify key factors such as:
  - Material balance: Compare the pieces on both sides.
  - Positioning: Evaluate the placement of pieces, control of the center, and potential threats.
  - Tactical opportunities: Look for immediate tactical shots like forks, pins, or discovered attacks.
  - Strategic considerations: Discuss long-term plans, weaknesses, and strengths of each side.
3. **Best Move Recommendation**: Propose several moves based on your analysis. Think of possible responses from the opponent and how to counteract them. Choose the best move that maximizes your advantage or minimizes your losses.
4. **Goal of the Puzzle**: Keep in mind that the primary objective is: {goal}. Tailor your analysis and move recommendations to align with this goal.
5. **Formatting and Clarity**: Provide your final answer in the following format: **Answer: <your move here>**, where your move is represented in UCI notation, e.g., e2e4, e1g1 (castling), e7e8q (promotion). Ensure your response is separated from the analysis in one line for clarity.

The puzzle is given by FEN string: {fen}",
        )
    }

    async fn call_openai_api(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = openai_api_rs::v1::chat_completion::ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![openai_api_rs::v1::chat_completion::ChatCompletionMessage {
                role: openai_api_rs::v1::chat_completion::MessageRole::user,
                content: prompt.to_string(),
                name: None,
                function_call: None,
            }],
            max_tokens: None,
            temperature: Some(0.5),
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            function_call: None,
            functions: None,
        };

        let response = self.client.chat_completion(request).await?;

        if let Some(choice) = response.choices.first() {
            if let Some(content) = &choice.message.content {
                Ok(content.to_string())
            } else {
                Err("No content in response".into())
            }
        } else {
            Err("No choices in response".into())
        }
    }
}

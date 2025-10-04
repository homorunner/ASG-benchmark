use serde::{Deserialize, Serialize};
use std::env;

use crate::puzzle::{Puzzle, PuzzleCollection, PuzzleGoal, PuzzleScore};

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
                let solution = solver.solve_puzzle(puzzle);
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

        // 计算游戏类型细分
        let mut game_type_scores: std::collections::HashMap<String, (usize, f64, f64)> =
            std::collections::HashMap::new();
        for score in &puzzle_scores {
            if let Some(puzzle) = self
                .puzzles
                .puzzles
                .iter()
                .find(|p| p.id == score.puzzle_id)
            {
                let entry = game_type_scores
                    .entry(puzzle.game_type.clone())
                    .or_insert((0, 0.0, 0.0));
                entry.0 += 1;
                entry.1 += score.score;
                entry.2 += score.max_possible_score;
            }
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

    pub fn solve_puzzle(&self, puzzle: &Puzzle) -> Vec<String> {
        let prompt = self.build_prompt(puzzle);

        // Since we can't use async in the trait method, we'll use tokio runtime
        let result = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(async { self.call_openai_api(&prompt).await });

        match result {
            Ok(response) => {
                // Parse the response into individual moves
                response.split(',').map(|s| s.trim().to_string()).collect()
            }
            Err(e) => {
                eprintln!("Error calling OpenAI API: {}", e);
                vec!["error".to_string()]
            }
        }
    }
}

impl Solver {
    pub fn new(model: String) -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let base_url =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let mut client = openai_api_rs::v1::api::Client::new(api_key);
        client.api_endpoint = base_url;

        Ok(Self {
            name: format!("OpenAI Solver ({})", model),
            description: format!("OpenAI API solver using {} model", model),
            model,
            client,
        })
    }

    fn build_prompt(&self, puzzle: &Puzzle) -> String {
        format!(
            "You are an expert at solving abstract board game puzzles. \n\
            Game Type: {}\n\
            Puzzle Goal: {}\n\
            Puzzle Description: {}\n\
            \n\
            Current Game State(s):\n\
            {}\n\
            \n\
            Please provide the best move(s) for this puzzle. Return only the move(s) in the format expected by the puzzle solution. \
            If there are multiple steps, provide them in order separated by commas.",
            puzzle.game_type,
            match &puzzle.goal {
                PuzzleGoal::FindBestMove => "Find the best move",
                PuzzleGoal::Custom(s) => s,
            },
            puzzle.description,
            puzzle.game_states.join("\n")
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
            max_tokens: Some(500),
            temperature: Some(0.1),
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
                Ok(content.trim().to_string())
            } else {
                Err("No content in response".into())
            }
        } else {
            Err("No choices in response".into())
        }
    }
}

# Abstract Strategy Game Benchmark

A benchmark for evaluating LLM performance on abstract strategy game puzzles, currently focused on board games like chess.

## Overview

BoardgameBench is designed to systematically evaluate how well Large Language Models (LLMs) can solve abstract board game puzzles across various games like chess, Go, xiangqi, quoridor, and more. The framework supports puzzles with single or multi-step solutions, allowing for nuanced scoring based on how far an LLM can progress through a puzzle sequence.

## Features

- **Multi-game Support**: Easily extensible to support various abstract board games
- **Text-based Puzzle Definition**: Puzzles defined using standard notation (e.g., FEN for chess)
- **Flexible Scoring**: Supports partial credit for multi-step puzzles
- **Comprehensive Results**: Detailed breakdown by game type and individual puzzle performance
- **JSON-based Configuration**: Simple puzzle and result formats for easy integration

## Project Structure

```
boardgamebench/
├── src/
│   ├── benchmark        # Benchmark application entry point
│   │   ├── main.rs
│   ├── generate         # Puzzle generation utility
│   │   ├── main.rs
│   ├── lib.rs           # Core library functionality
│   ├── game.rs          # Game definitions and rules
│   ├── puzzle.rs        # Puzzle data structures
│   └── evaluation.rs    # Benchmark runner and scoring logic
├── sample_puzzles.json  # Example puzzle collection
├── Cargo.toml           # Rust project configuration
```

## Quick Start

### Prerequisites

- Rust toolchain (latest stable version)

### Usage

The benchmark will automatically load puzzles from `sample_puzzles.json` and run them against a test solver:

```bash
# Run with default configuration
cargo run
```

## Puzzle Format

Puzzles are defined in JSON format with the following structure:

```json
{
  "name": "Collection Name",
  "description": "Collection description",
  "puzzles": [
    {
      "id": "puzzle_id",
      "game_type": "chess",
      "description": "Puzzle description",
      "goal": {"FindBestMove": null},
      "game_states": ["fen_string_1", "fen_string_2"],
      "solutions": ["move1", "move2"]
    }
  ]
}
```

### Supported Game Types

Currently supported:
- **Chess**: Using FEN notation for board states

Should be easily extensible for other abstract board games

## API Usage

### Basic Integration

```rust
use boardgamebench::{PuzzleCollection, BenchmarkRunner, TextSolver};

// Load puzzles
let puzzles = PuzzleCollection::load_from_file("puzzles.json")?;

// Create solver
let solver = TextSolver {
    name: "MySolver".to_string(),
    description: "Custom LLM solver".to_string(),
};

// Run benchmark
let runner = BenchmarkRunner::new(puzzles);
let results = runner.run_benchmark(&solver);

// Export results
runner.export_results(&results, "results.json")?;
```

### Custom Solvers

Implement the `Solver` trait to create custom solvers:

```rust
use boardgamebench::evaluation::Solver;

struct MyCustomSolver;

impl Solver for MyCustomSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        // Custom solving logic here
        vec![]
    }
}
```

## Results Format

Benchmark results include:
- Overall score and percentage
- Breakdown by game type
- Individual puzzle performance
- Detailed scoring information

Results are exported to JSON for further analysis:

```json
{
  "benchmark_name": "Sample Chess Puzzles Collection",
  "solver_name": "TestSolver",
  "total_score": 2,
  "max_possible_score": 2,
  "average_score": 1.0,
  "game_type_breakdown": [...],
  "puzzle_scores": [...]
}
```

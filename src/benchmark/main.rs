use anyhow::Result;
use clap::Parser;

use boardgamebench::evaluation::{BenchmarkRunner, Solver};
use boardgamebench::puzzle::PuzzleCollection;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Model name in OpenAI API
    #[arg(short, long, default_value = "deepseek-chat")]
    model: String,

    /// Puzzle file to load
    #[arg(short, long, default_value = "sample_puzzles.json")]
    puzzle_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let puzzles = PuzzleCollection::load_from_file(&args.puzzle_file)?;
    println!(
        "Loaded {} puzzles from collection: {}",
        puzzles.puzzles.len(),
        puzzles.name
    );

    dotenvy::dotenv().ok();

    let solver: Box<boardgamebench::evaluation::Solver> = {
        println!("Using Solver with model: {}", args.model);
        match Solver::new(args.model) {
            Ok(solver) => Box::new(solver),
            Err(e) => {
                eprintln!("Failed to create solver: {}", e);
                return Err(anyhow::anyhow!("Failed to create solver: {}", e));
            }
        }
    };

    let runner = BenchmarkRunner::new(puzzles);
    let results = runner.run_benchmark(solver.as_ref());

    println!("\nBenchmark Results:");
    println!("Benchmark: {}", results.benchmark_name);
    println!(
        "Solver: {} - {}",
        results.solver_name, results.solver_description
    );
    println!("Total Puzzles: {}", results.total_puzzles);
    println!(
        "Total Score: {}/{}",
        results.total_score, results.max_possible_score
    );
    println!("Scoring average: {:.2}%", results.average_score * 100.0);

    println!("\nGame Type Breakdown:");
    for game_type in &results.game_type_breakdown {
        println!(
            "  {}: {:.2}% ({} puzzles)",
            game_type.game_type,
            game_type.average_score * 100.0,
            game_type.count
        );
    }

    println!("\nIndividual Puzzle Results:");
    for score in &results.puzzle_scores {
        let status = if score.max_possible_score == score.score {
            "✅"
        } else {
            "❌"
        };
        println!(
            "  {} {}: {}/{}",
            status, score.puzzle_id, score.score, score.max_possible_score
        );
    }

    if let Err(e) = runner.export_results(&results, "benchmark_results.json") {
        eprintln!("Warning: Could not export results: {}", e);
    } else {
        println!("\nResults exported to benchmark_results.json");
    }

    Ok(())
}

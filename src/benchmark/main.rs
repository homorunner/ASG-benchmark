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
    #[arg(short, long, default_value = "data/sample_puzzles.json")]
    puzzle_file: String,

    /// Number of threads for parallel evaluation
    #[arg(short, long, default_value = "16")]
    threads: usize,

    /// Number of passes to run for each test case
    #[arg(short = 'N', long, default_value = "1")]
    passes: usize,
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
            Ok(solver) => {
                // Test API reachability before running benchmark
                println!("Testing API reachability...");
                match solver.test_api_reachability() {
                    Ok(response) => {
                        println!("API test successful. Response: {}", response);
                        Box::new(solver)
                    }
                    Err(e) => {
                        eprintln!("API test failed: {}", e);
                        eprintln!("Please check your OPENAI_API_KEY and OPENAI_BASE_URL environment variables.");
                        return Err(anyhow::anyhow!("API reachability test failed: {}", e));
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create solver: {}", e);
                return Err(anyhow::anyhow!("Failed to create solver: {}", e));
            }
        }
    };

    println!("Using {} threads for parallel evaluation", args.threads);
    println!("Running {} passes for each test case", args.passes);
    let runner = BenchmarkRunner::new(puzzles);

    let results = if args.passes > 1 {
        runner.run_benchmark_multiple_passes(solver.as_ref(), args.threads, args.passes)
    } else {
        runner.run_benchmark_parallel(solver.as_ref(), args.threads)
    };

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

    // Display pass@1 and pass@n results if multiple passes were run
    if args.passes > 1 {
        if let Some(pass_results) = &results.pass_results {
            println!("\nResults:");
            println!("  Pass@1: {:.2}%", pass_results.pass_at_1 * 100.0);
            println!("  Pass@{}: {:.2}%", args.passes, pass_results.pass_at_n * 100.0);
        }
    }

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

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

use boardgamebench::puzzle::{Puzzle, PuzzleCollection};

#[derive(Debug, Clone)]
struct PuzzleData {
    rating: f64,
    fen: String,
    moves: String,
    game_url: String,
    themes: String,
}

fn read_puzzle_database(file_path: &str) -> Result<Vec<PuzzleData>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut puzzles = Vec::new();

    for line in reader.lines().skip(1) {
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() >= 9 {
            let fen = fields[1].to_string();
            let moves = fields[2].to_string();
            let rating = fields[3].parse::<f64>().unwrap_or(0.0);
            let _popularity = fields[5].parse::<f64>().unwrap_or(0.0);
            let _nb_plays = fields[6].parse::<f64>().unwrap_or(0.0);
            let themes = fields[7].to_string();
            let game_url = fields[8].to_string();

            puzzles.push(PuzzleData {
                rating,
                fen,
                moves,
                game_url,
                themes,
            });
        }
    }

    Ok(puzzles)
}

fn filter_puzzles_by_theme(
    puzzles: &[PuzzleData],
    theme: &str,
    _min_popularity: f64,
    _min_plays: f64,
    rating_range: (f64, f64),
) -> Vec<PuzzleData> {
    puzzles
        .iter()
        .filter(|p| {
            p.themes.contains(theme)
                && p.rating >= rating_range.0
                && p.rating <= rating_range.1
        })
        .cloned()
        .collect()
}

fn generate_puzzles_from_data(
    puzzle_data: &[PuzzleData],
    puzzle_type: &str,
    count: usize,
    seed: u64,
) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut selected_puzzles: Vec<&PuzzleData> = puzzle_data.iter().collect();
    selected_puzzles = selected_puzzles.partial_shuffle(&mut rng, count).0.to_vec();

    let mut puzzles = Vec::new();

    for (i, puzzle) in selected_puzzles.iter().enumerate() {
        let moves: Vec<&str> = puzzle.moves.split_whitespace().collect();

        // Calculate the FEN after the first move
        let pos = Chess::from_setup(
            Setup::from(Fen::from_ascii(puzzle.fen.as_bytes())?),
            CastlingMode::Standard,
        )?;
        let move0: UciMove = moves[0].parse()?;
        let chess_move = move0.to_move(&pos)?;
        let pos_after_move = pos.play(chess_move)?;
        let fen_after_move = Fen::from_position(&pos_after_move, EnPassantMode::Always);

        let move1 = moves[1].to_string();

        let puzzle_obj = Puzzle {
            id: format!("chess_{}_{:02}", puzzle_type, i + 1),
            description: format!("Chess {} puzzle from {}", puzzle_type, puzzle.game_url),
            game_states: vec![fen_after_move.to_string()],
            solutions: vec![move1],
        };

        puzzles.push(puzzle_obj);
    }

    Ok(puzzles)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read the puzzle database
    let all_puzzles = read_puzzle_database("database/lichess_db_puzzle.csv")?;
    println!("Loaded {} puzzles from database", all_puzzles.len());

    // Define puzzle types and their criteria
    let puzzle_types = vec![
        ("mateIn1", (1000.0, 1500.0)),
        ("opening", (1200.0, 1800.0)),
        ("middlegame", (1400.0, 2000.0)),
        ("endgame", (1600.0, 2200.0)),
    ];

    let mut all_generated_puzzles = Vec::new();

    for (theme, rating_range) in puzzle_types {
        println!("Generating {} puzzles...", theme);

        // Filter puzzles by theme and rating
        let filtered_puzzles = filter_puzzles_by_theme(
            &all_puzzles,
            theme,
            99.0,  // min_popularity
            500.0, // min_plays
            rating_range,
        );

        println!("Found {} {} puzzles", filtered_puzzles.len(), theme);

        // Generate 10 puzzles for this type
        let puzzles = generate_puzzles_from_data(&filtered_puzzles, theme, 10, 3407)?;
        all_generated_puzzles.extend(puzzles);
    }

    // Create the puzzle collection
    let collection = PuzzleCollection {
        name: "Lichess Multi-Type Chess Puzzles Collection".to_string(),
        description: "A collection of chess puzzles including mate-in-1, opening, middlegame, and endgame positions extracted from Lichess database".to_string(),
        game_type: "chess".to_string(),
        goal: "Find the best move to win for current player in the given chess game.".to_string(),
        puzzles: all_generated_puzzles,
    };

    // Save the collection
    let json_output = serde_json::to_string_pretty(&collection)?;
    let mut output_file = File::create("data/lichess_multi_type_puzzles.json")?;
    write!(output_file, "{}", json_output)?;

    println!("Successfully generated lichess_multi_type_puzzles.json");
    println!("Generated {} puzzles", collection.puzzles.len());

    Ok(())
}

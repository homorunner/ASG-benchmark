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

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("database/lichess_db_puzzle.csv")?;
    let reader = BufReader::new(file);

    // Collect puzzles with mateIn1 theme
    let mut mate_in_1_puzzles: Vec<(f64, String, String, String, String)> = Vec::new();

    for line in reader.lines().skip(1) {
        // Skip header
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() >= 9 {
            let puzzle_id = fields[0].to_string();
            let fen = fields[1].to_string();
            let moves = fields[2].to_string();
            let rating = fields[3].parse::<f64>().unwrap();
            let popularity = fields[5].parse::<f64>().unwrap();
            let nb_plays = fields[6].parse::<f64>().unwrap();
            let themes = fields[7].to_string();
            let game_url = fields[8].to_string();

            if themes.contains("mateIn1") && popularity > 99. && nb_plays > 500. && (rating > 1000. && rating < 1500.) {
                mate_in_1_puzzles.push((rating, fen, moves, game_url, puzzle_id));
            }
        }
    }

    // Take 20 random puzzles
    let mut rng = SmallRng::seed_from_u64(3407);
    mate_in_1_puzzles = mate_in_1_puzzles.partial_shuffle(&mut rng, 10).0.to_vec();

    let mut puzzles = Vec::new();

    for (i, (_rating, fen, moves, game_url, _puzzle_id)) in mate_in_1_puzzles.iter().enumerate() {
        let moves: Vec<&str> = moves.split_whitespace().collect();

        // Calculate the FEN after the first move
        let pos = Chess::from_setup(
            Setup::from(Fen::from_ascii(fen.as_bytes())?),
            CastlingMode::Standard,
        )?;
        let move0: UciMove = moves[0].parse()?;
        let chess_move = move0.to_move(&pos)?;
        let pos_after_move = pos.play(chess_move)?;
        let fen_after_move = Fen::from_position(&pos_after_move, EnPassantMode::Always);

        let move1 = moves[1].to_string();

        let puzzle_obj = Puzzle {
            id: format!("chess_mate_in_1_{:02}", i + 1),
            description: format!("Chess puzzle from {}", game_url),
            game_states: vec![fen_after_move.to_string()],
            solutions: vec![move1],
        };

        puzzles.push(puzzle_obj);
    }

    let collection = PuzzleCollection {
        name: "Lichess Mate-in-1 Puzzles Collection".to_string(),
        description: "A collection of mate-in-1 chess puzzles extracted from Lichess database"
            .to_string(),
        game_type: "chess".to_string(),
        goal: "Find the best move to win for current player in the given chess game.".to_string(),
        puzzles,
    };

    let json_output = serde_json::to_string_pretty(&collection)?;
    let mut output_file = File::create("lichess_mate_in_1_puzzles.json")?;
    write!(output_file, "{}", json_output)?;

    println!("Successfully generated lichess_mate_in_1_puzzles.json");
    println!("Generated {} puzzles", collection.puzzles.len());

    Ok(())
}

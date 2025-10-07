use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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
            let rating_str = fields[3];
            let themes = fields[7].to_string();
            let game_url = fields[8].to_string();

            if let Ok(rating) = rating_str.parse::<f64>() {
                if themes.contains("mateIn1") {
                    mate_in_1_puzzles.push((rating, fen, moves, game_url, puzzle_id));
                }
            }
        }
    }

    mate_in_1_puzzles.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let top_10_puzzles: Vec<(f64, String, String, String, String)> =
        mate_in_1_puzzles.into_iter().take(10).collect();

    let mut puzzles = Vec::new();

    for (i, (_rating, fen, moves, game_url, _puzzle_id)) in top_10_puzzles.iter().enumerate() {
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
            game_type: "chess".to_string(),
            description: format!("Chess puzzle from {}", game_url),
            goal: "按照标准国际象棋规则，当前局面下正在行棋的玩家走哪一步棋可以最快获胜？谜题保证答案是唯一的，请找到这一步棋，以UCI格式输出，例如b1c3、e1g1(白方短易位)、e7e8q(升变)。".to_string(),
            game_states: vec![fen_after_move.to_string()],
            solutions: vec![move1],
        };

        puzzles.push(puzzle_obj);
    }

    let collection = PuzzleCollection {
        name: "Lichess Mate-in-1 Puzzles Collection".to_string(),
        description: "A collection of mate-in-1 chess puzzles extracted from Lichess database"
            .to_string(),
        puzzles,
    };

    let json_output = serde_json::to_string_pretty(&collection)?;
    let mut output_file = File::create("lichess_mate_in_1_puzzles.json")?;
    write!(output_file, "{}", json_output)?;

    println!("Successfully generated lichess_mate_in_1_puzzles.json");
    println!("Generated {} puzzles", collection.puzzles.len());

    Ok(())
}

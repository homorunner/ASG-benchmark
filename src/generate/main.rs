use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::*;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use rand::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use image::open;

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

fn load_board_themes() -> Result<Vec<String>, Box<dyn Error>> {
    let board_dir = Path::new("images/chess/board");
    let mut themes = Vec::new();

    if board_dir.exists() && board_dir.is_dir() {
        for entry in fs::read_dir(board_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "png") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    themes.push(stem.to_string());
                }
            }
        }
    }

    println!("Loaded {} board themes: {:?}", themes.len(), themes);
    Ok(themes)
}

fn load_piece_styles() -> Result<Vec<String>, Box<dyn Error>> {
    let pieces_dir = Path::new("images/chess/pieces");
    let mut styles = Vec::new();

    if pieces_dir.exists() && pieces_dir.is_dir() {
        for entry in fs::read_dir(pieces_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(style_name) = path.file_name().and_then(|s| s.to_str()) {
                    styles.push(style_name.to_string());
                }
            }
        }
    }

    println!("Loaded {} piece styles: {:?}", styles.len(), styles);
    Ok(styles)
}

/// Generate a chess board image from FEN notation using random board and piece themes
fn generate_board_image_from_fen(
    fen: &str,
    output_path: &str,
    seed: u64,
) -> Result<(), Box<dyn Error>> {
    let mut rng = SmallRng::seed_from_u64(seed);

    let board_themes = load_board_themes()?;
    if board_themes.is_empty() {
        return Err("No board themes found in images/chess/board/".into());
    }

    let piece_styles = load_piece_styles()?;
    if piece_styles.is_empty() {
        return Err("No piece styles found in images/chess/pieces/".into());
    }

    // Randomly select board theme and piece style
    let board_theme = board_themes.choose(&mut rng).unwrap();
    let piece_style = piece_styles.choose(&mut rng).unwrap();

    println!("Generating board image with theme '{}' and piece style '{}'", board_theme, piece_style);

    let board_path = format!("images/chess/board/{}.png", board_theme);
    let mut board_image = open(&board_path)?;

    let pos = Chess::from_setup(
        Setup::from(Fen::from_ascii(fen.as_bytes())?),
        CastlingMode::Standard,
    )?;

    let board = pos.board();

    let square_size = 150;
    let board_offset_x = 0;
    let board_offset_y = 0;
    
    board_image = board_image.resize(
        square_size * 8 + board_offset_x * 2,
        square_size * 8 + board_offset_y * 2,
        image::imageops::FilterType::Gaussian);

    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::from_coords(shakmaty::File::new(file), shakmaty::Rank::new(7 - rank)); // Convert to chess coordinates (a1 is bottom-left)
            if let Some(piece) = board.piece_at(square) {
                let piece_code = match (piece.color, piece.role) {
                    (Color::White, Role::Pawn) => "wp",
                    (Color::White, Role::Knight) => "wn",
                    (Color::White, Role::Bishop) => "wb",
                    (Color::White, Role::Rook) => "wr",
                    (Color::White, Role::Queen) => "wq",
                    (Color::White, Role::King) => "wk",
                    (Color::Black, Role::Pawn) => "bp",
                    (Color::Black, Role::Knight) => "bn",
                    (Color::Black, Role::Bishop) => "bb",
                    (Color::Black, Role::Rook) => "br",
                    (Color::Black, Role::Queen) => "bq",
                    (Color::Black, Role::King) => "bk",
                };

                let piece_path = format!("images/chess/pieces/{}/{}.png", piece_style, piece_code);
                let piece_image = open(&piece_path)?;

                let x = board_offset_x + (file as u32) * square_size;
                let y = board_offset_y + (rank as u32) * square_size;

                image::imageops::overlay(&mut board_image, &piece_image, x as i64, y as i64);
            }
        }
    }

    board_image.save(output_path)?;
    println!("Board image saved to: {}", output_path);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read the puzzle database
    let all_puzzles = read_puzzle_database("database/lichess_db_puzzle.csv")?;
    println!("Loaded {} puzzles from database", all_puzzles.len());

    // Define puzzle types and their criteria
    let puzzle_types = vec![
        ("opening", (1200.0, 1800.0)),
        ("middlegame", (1200.0, 1500.0)),
        ("endgame", (1200.0, 1800.0)),
        ("defensiveMove", (1200.0, 1500.0)),
        ("quietMove", (1200.0, 1500.0)),
    ];

    let mut all_generated_puzzles = Vec::new();

    for (theme, rating_range) in puzzle_types {
        println!("Generating {} puzzles...", theme);

        // Filter puzzles by theme and rating
        let filtered_puzzles = filter_puzzles_by_theme(
            &all_puzzles,
            theme,
            99.0,  // min_popularity
            1000.0, // min_plays
            rating_range,
        );

        println!("Found {} {} puzzles", filtered_puzzles.len(), theme);

        // Generate 10 puzzles for this type
        let puzzles = generate_puzzles_from_data(&filtered_puzzles, theme, 20, 3407)?;
        all_generated_puzzles.extend(puzzles);
    }

    // Create the puzzle collection
    let collection = PuzzleCollection {
        name: "Lichess Multi-Type Chess Puzzles Collection".to_string(),
        description: "A collection of chess puzzles including mate-in-1, opening, middlegame, and endgame positions extracted from Lichess database".to_string(),
        game_type: "chess".to_string(),
        goal: "Find the best move to win for current player in the given chess game.".to_string(),
        game_rule: "".to_string(),
        puzzles: all_generated_puzzles,
    };

    // Save the collection
    let json_output = serde_json::to_string_pretty(&collection)?;
    let mut output_file = File::create("data/lichess_multi_type_puzzles.json")?;
    write!(output_file, "{}", json_output)?;

    println!("Successfully generated lichess_multi_type_puzzles.json");
    println!("Generated {} puzzles", collection.puzzles.len());

    // Test the board image generation function
    println!("\nTesting board image generation...");
    let test_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; // Standard starting position
    let output_path = "output/images/test_board.png";
    generate_board_image_from_fen(test_fen, output_path, 12345)?;
    println!("Test board image generated successfully!");

    Ok(())
}

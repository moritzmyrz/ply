use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use ply::export::{to_json_aggregate, to_json_summary};
use ply::fen::{parse_fen, to_fen};
use ply::movegen::generate_legal_moves;
use ply::pgn::{parse_pgn, reconstruct_game};
use ply::stats::{aggregate_stats, summarize_game, summarize_games};

#[derive(Parser, Debug)]
#[command(name = "ply")]
#[command(about = "Rust chess toolkit for parsing and analysis")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Validate {
        file: PathBuf,
    },
    Summarize {
        file: PathBuf,
    },
    Stats {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Fen {
        fen: String,
        #[arg(long)]
        legal_moves: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Validate { file } => cmd_validate(&file),
        Commands::Summarize { file } => cmd_summarize(&file),
        Commands::Stats { file, json } => cmd_stats(&file, json),
        Commands::Fen { fen, legal_moves } => cmd_fen(&fen, legal_moves),
    };
    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn cmd_validate(file: &PathBuf) -> Result<(), String> {
    let content = fs::read_to_string(file).map_err(|e| format!("failed to read file: {e}"))?;
    let games = parse_pgn(&content).map_err(|e| format!("{e:?}"))?;
    let mut ok = 0usize;
    let mut failed = 0usize;
    for game in &games {
        match reconstruct_game(game) {
            Ok(_) => ok += 1,
            Err(_) => failed += 1,
        }
    }
    println!("validated games: {}", games.len());
    println!("valid: {ok}");
    println!("invalid: {failed}");
    Ok(())
}

fn cmd_summarize(file: &PathBuf) -> Result<(), String> {
    let content = fs::read_to_string(file).map_err(|e| format!("failed to read file: {e}"))?;
    let games = parse_pgn(&content).map_err(|e| format!("{e:?}"))?;
    for (idx, game) in games.iter().enumerate() {
        match reconstruct_game(game) {
            Ok(record) => {
                let s = summarize_game(&record);
                println!(
                    "{}. {} vs {} | result={} | plies={}",
                    idx + 1,
                    s.white.as_deref().unwrap_or("?"),
                    s.black.as_deref().unwrap_or("?"),
                    s.result.as_deref().unwrap_or("*"),
                    s.plies
                );
            }
            Err(err) => {
                println!("{}. invalid game: {err:?}", idx + 1);
            }
        }
    }
    Ok(())
}

fn cmd_stats(file: &PathBuf, json: bool) -> Result<(), String> {
    let content = fs::read_to_string(file).map_err(|e| format!("failed to read file: {e}"))?;
    let games = parse_pgn(&content).map_err(|e| format!("{e:?}"))?;
    let records = games
        .iter()
        .filter_map(|g| reconstruct_game(g).ok())
        .collect::<Vec<_>>();
    let summaries = summarize_games(&records);
    let stats = aggregate_stats(&summaries);
    if json {
        let payload = serde_json::json!({
            "stats": to_json_aggregate(&stats),
            "games": summaries.iter().map(to_json_summary).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| format!("json error: {e}"))?
        );
    } else {
        println!("games: {}", stats.games);
        println!("white_wins: {}", stats.white_wins);
        println!("black_wins: {}", stats.black_wins);
        println!("draws: {}", stats.draws);
        println!("unresolved: {}", stats.unresolved);
        println!("total_plies: {}", stats.total_plies);
        println!("average_plies: {:.2}", stats.average_plies);
    }
    Ok(())
}

fn cmd_fen(fen: &str, legal_moves: bool) -> Result<(), String> {
    let position = parse_fen(fen).map_err(|e| format!("{e:?}"))?;
    println!("normalized_fen: {}", to_fen(&position));
    if legal_moves {
        let moves = generate_legal_moves(&position);
        let rendered = moves
            .iter()
            .map(|m| format!("{}{}", m.from.to_algebraic(), m.to.to_algebraic()))
            .collect::<Vec<_>>();
        println!("legal_moves ({}): {}", rendered.len(), rendered.join(" "));
    }
    Ok(())
}

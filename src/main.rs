use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, Subcommand};
use ply::export::{to_json_aggregate, to_json_summary};
use ply::fen::{parse_fen, to_fen, STARTPOS_FEN};
use ply::movegen::generate_legal_moves;
use ply::perft::perft;
use ply::pgn::{parse_pgn, reconstruct_game};
use ply::stats::{aggregate_record_stats, summarize_game, summarize_games};

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
    Perft {
        #[arg(long)]
        fen: Option<String>,
        #[arg(long)]
        depth: u8,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Validate { file } => cmd_validate(&file),
        Commands::Summarize { file } => cmd_summarize(&file),
        Commands::Stats { file, json } => cmd_stats(&file, json),
        Commands::Fen { fen, legal_moves } => cmd_fen(&fen, legal_moves),
        Commands::Perft { fen, depth } => cmd_perft(fen.as_deref(), depth),
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
    let records = games.iter().filter_map(|g| reconstruct_game(g).ok()).collect::<Vec<_>>();
    let summaries = summarize_games(&records);
    let stats = aggregate_record_stats(&records);
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
        println!("results:");
        println!("  white_wins: {}", stats.white_wins);
        println!("  black_wins: {}", stats.black_wins);
        println!("  draws: {}", stats.draws);
        println!("  unresolved: {}", stats.unresolved);

        println!("length:");
        println!("  total_plies: {}", stats.total_plies);
        println!("  average_plies: {:.2}", stats.average_plies);
        println!("  average_plies_white_wins: {}", format_optional(stats.average_plies_white_wins));
        println!("  average_plies_black_wins: {}", format_optional(stats.average_plies_black_wins));
        println!("  average_plies_draws: {}", format_optional(stats.average_plies_draws));
        println!("  average_plies_unresolved: {}", format_optional(stats.average_plies_unresolved));

        println!("first_moves_white:");
        for (mv, count) in &stats.white_first_moves {
            println!("  {mv}: {count}");
        }
        println!("first_moves_black:");
        for (mv, count) in &stats.black_first_moves {
            println!("  {mv}: {count}");
        }

        println!("castling:");
        println!("  games_with_kingside_castle: {}", stats.games_with_kingside_castle);
        println!("  games_with_queenside_castle: {}", stats.games_with_queenside_castle);
        println!("  games_with_no_castling: {}", stats.games_with_no_castling);

        println!("move_events:");
        println!("  total_captures: {}", stats.total_captures);
        println!("  average_captures: {:.2}", stats.average_captures);
        println!("  total_checks: {}", stats.total_checks);
        println!("  average_checks: {:.2}", stats.average_checks);
        println!("  total_promotions: {}", stats.total_promotions);
        println!("  average_promotions: {:.2}", stats.average_promotions);
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

fn cmd_perft(fen: Option<&str>, depth: u8) -> Result<(), String> {
    let fen = fen.unwrap_or(STARTPOS_FEN);
    let position = parse_fen(fen).map_err(|e| format!("{e:?}"))?;

    let started = Instant::now();
    let nodes = perft(&position, depth);
    let elapsed = started.elapsed();
    let secs = elapsed.as_secs_f64();
    let nps = if secs > 0.0 { (nodes as f64 / secs) as u64 } else { 0 };

    println!("depth: {depth}");
    println!("nodes: {nodes}");
    println!("elapsed_ms: {}", elapsed.as_millis());
    println!("nps: {nps}");
    Ok(())
}

fn format_optional(value: Option<f64>) -> String {
    value.map(|v| format!("{v:.2}")).unwrap_or_else(|| "-".to_string())
}

pub mod commands;
pub mod render;

use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use ply::fen::{parse_fen, to_fen, FenError, STARTPOS_FEN};
use ply::movegen::generate_legal_moves;
use ply::perft::{perft, perft_divide};
use ply::pgn::{reconstruct_game, PgnError, PgnReader};
use ply::stats::{summarize_game, AggregateStatsAccumulator};
use thiserror::Error;

use self::commands::{Cli, Commands};
use self::render::{
    render_fen, render_perft, render_stats, render_summaries, render_validate, FenOutput,
    PerftOutput, StatsOutput, SummariesOutput, SummaryEntry, ValidateFailure, ValidateOutput,
};

#[derive(Debug, Error)]
pub enum CliError {
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Fen(#[from] FenError),
    #[error(transparent)]
    Pgn(#[from] PgnError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Validate { file, verbose } => {
            print!("{}", render_validate(&cmd_validate(&file, verbose)?))
        }
        Commands::Summarize { file } => print!("{}", render_summaries(&cmd_summarize(&file)?)),
        Commands::Stats { file, json } => print!("{}", render_stats(&cmd_stats(&file, json)?)?),
        Commands::Fen { fen, legal_moves } => {
            print!("{}", render_fen(&cmd_fen(&fen, legal_moves)?))
        }
        Commands::Perft { fen, depth, divide } => {
            print!("{}", render_perft(&cmd_perft(fen.as_deref(), depth, divide)?))
        }
    }
    Ok(())
}

fn cmd_validate(file: &std::path::Path, verbose: bool) -> Result<ValidateOutput, CliError> {
    let reader = BufReader::new(File::open(file)?);
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut total = 0usize;
    let mut failures = Vec::new();
    for (idx, game) in PgnReader::new(reader).enumerate() {
        total += 1;
        let game = game?;
        match reconstruct_game(&game) {
            Ok(_) => ok += 1,
            Err(err) => {
                failed += 1;
                if verbose {
                    failures
                        .push(ValidateFailure { game_index: idx + 1, message: err.to_string() });
                }
            }
        }
    }
    Ok(ValidateOutput { validated_games: total, valid: ok, invalid: failed, failures })
}

fn cmd_summarize(file: &std::path::Path) -> Result<SummariesOutput, CliError> {
    let reader = BufReader::new(File::open(file)?);
    let mut entries = Vec::new();
    for (idx, game) in PgnReader::new(reader).enumerate() {
        match reconstruct_game(&game?) {
            Ok(record) => entries
                .push(SummaryEntry::Valid { index: idx + 1, summary: summarize_game(&record) }),
            Err(err) => {
                entries.push(SummaryEntry::Invalid { index: idx + 1, error: err.to_string() })
            }
        }
    }
    Ok(SummariesOutput { entries })
}

fn cmd_stats(file: &std::path::Path, json: bool) -> Result<StatsOutput, CliError> {
    let reader = BufReader::new(File::open(file)?);
    let mut acc = AggregateStatsAccumulator::default();
    let mut summaries = Vec::new();

    for game in PgnReader::new(reader) {
        let game = game?;
        if let Ok(record) = reconstruct_game(&game) {
            if json {
                let summary = summarize_game(&record);
                acc.push_record_with_summary(&record, &summary);
                summaries.push(summary);
            } else {
                acc.push_record(&record);
            }
        }
    }

    Ok(StatsOutput { json, stats: acc.finish(), summaries })
}

fn cmd_fen(fen: &str, legal_moves: bool) -> Result<FenOutput, CliError> {
    let position = parse_fen(fen)?;
    let rendered = if legal_moves {
        generate_legal_moves(&position).into_iter().map(|m| m.to_coordinate()).collect()
    } else {
        Vec::new()
    };
    Ok(FenOutput { normalized_fen: to_fen(&position), legal_moves: rendered })
}

fn cmd_perft(fen: Option<&str>, depth: u8, divide: bool) -> Result<PerftOutput, CliError> {
    let fen = fen.unwrap_or(STARTPOS_FEN);
    let position = parse_fen(fen)?;

    let started = Instant::now();
    let nodes = perft(&position, depth);
    let elapsed = started.elapsed();
    let secs = elapsed.as_secs_f64();
    let nps = if secs > 0.0 { (nodes as f64 / secs) as u64 } else { 0 };
    let divide = if divide {
        Some(
            perft_divide(&position, depth)
                .into_iter()
                .map(|(mv, count)| (mv.to_coordinate(), count))
                .collect(),
        )
    } else {
        None
    };

    Ok(PerftOutput { depth, nodes, elapsed_ms: elapsed.as_millis(), nps, divide })
}

use std::collections::BTreeMap;

use crate::board::Color;
use crate::fen::parse_fen;
use crate::movegen::{apply_move, is_in_check};
use crate::pgn::{color_from_result, GameRecord, PgnGame, OFFICIAL_STARTPOS_FEN};

#[derive(Debug, Clone)]
pub struct GameSummary {
    pub event: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    pub plies: usize,
    pub winner: Option<Color>,
}

#[derive(Debug, Clone, Default)]
pub struct AggregateStats {
    pub games: usize,
    pub white_wins: usize,
    pub black_wins: usize,
    pub draws: usize,
    pub unresolved: usize,
    pub total_plies: usize,
    pub average_plies: f64,
    pub white_first_moves: BTreeMap<String, usize>,
    pub black_first_moves: BTreeMap<String, usize>,
    pub games_with_kingside_castle: usize,
    pub games_with_queenside_castle: usize,
    pub games_with_no_castling: usize,
    pub total_captures: usize,
    pub average_captures: f64,
    pub total_checks: usize,
    pub average_checks: f64,
    pub total_promotions: usize,
    pub average_promotions: f64,
    pub average_plies_white_wins: Option<f64>,
    pub average_plies_black_wins: Option<f64>,
    pub average_plies_draws: Option<f64>,
    pub average_plies_unresolved: Option<f64>,
}

pub fn summarize_game(record: &GameRecord) -> GameSummary {
    let tags = &record.game.tags;
    let result = record.game.result.clone().or_else(|| tags.get("Result").cloned());
    let winner = color_from_result(result.as_ref());

    GameSummary {
        event: tags.get("Event").cloned(),
        white: tags.get("White").cloned(),
        black: tags.get("Black").cloned(),
        result,
        plies: record.plies.len(),
        winner,
    }
}

pub fn summarize_games(records: &[GameRecord]) -> Vec<GameSummary> {
    records.iter().map(summarize_game).collect()
}

pub fn aggregate_stats(summaries: &[GameSummary]) -> AggregateStats {
    let mut stats = AggregateStats { games: summaries.len(), ..AggregateStats::default() };
    let mut white_wins_plies = 0usize;
    let mut black_wins_plies = 0usize;
    let mut draws_plies = 0usize;
    let mut unresolved_plies = 0usize;

    for summary in summaries {
        stats.total_plies += summary.plies;
        match summary.result.as_deref() {
            Some("1-0") => {
                stats.white_wins += 1;
                white_wins_plies += summary.plies;
            }
            Some("0-1") => {
                stats.black_wins += 1;
                black_wins_plies += summary.plies;
            }
            Some("1/2-1/2") => {
                stats.draws += 1;
                draws_plies += summary.plies;
            }
            _ => {
                stats.unresolved += 1;
                unresolved_plies += summary.plies;
            }
        }
    }
    if stats.games > 0 {
        stats.average_plies = stats.total_plies as f64 / stats.games as f64;
    }
    stats.average_plies_white_wins = average_if_nonzero(white_wins_plies, stats.white_wins);
    stats.average_plies_black_wins = average_if_nonzero(black_wins_plies, stats.black_wins);
    stats.average_plies_draws = average_if_nonzero(draws_plies, stats.draws);
    stats.average_plies_unresolved = average_if_nonzero(unresolved_plies, stats.unresolved);
    stats
}

pub fn aggregate_record_stats(records: &[GameRecord]) -> AggregateStats {
    let summaries = summarize_games(records);
    let mut stats = aggregate_stats(&summaries);

    for record in records {
        if let Some(first_white) = record.game.moves.first() {
            *stats.white_first_moves.entry(first_white.clone()).or_insert(0) += 1;
        }
        if let Some(first_black) = record.game.moves.get(1) {
            *stats.black_first_moves.entry(first_black.clone()).or_insert(0) += 1;
        }

        let mut saw_kingside = false;
        let mut saw_queenside = false;
        let mut position = initial_position_for_record(record);

        for mv in &record.plies {
            if mv.is_capture {
                stats.total_captures += 1;
            }
            if mv.promotion.is_some() {
                stats.total_promotions += 1;
            }
            match mv.castle {
                Some(crate::board::CastleSide::KingSide) => saw_kingside = true,
                Some(crate::board::CastleSide::QueenSide) => saw_queenside = true,
                None => {}
            }

            apply_move(&mut position, *mv);
            if is_in_check(&position, position.side_to_move) {
                stats.total_checks += 1;
            }
        }

        if saw_kingside {
            stats.games_with_kingside_castle += 1;
        }
        if saw_queenside {
            stats.games_with_queenside_castle += 1;
        }
        if !saw_kingside && !saw_queenside {
            stats.games_with_no_castling += 1;
        }
    }

    if stats.games > 0 {
        let games = stats.games as f64;
        stats.average_captures = stats.total_captures as f64 / games;
        stats.average_checks = stats.total_checks as f64 / games;
        stats.average_promotions = stats.total_promotions as f64 / games;
    }

    stats
}

pub fn summarize_raw_game(game: &PgnGame) -> GameSummary {
    GameSummary {
        event: game.tags.get("Event").cloned(),
        white: game.tags.get("White").cloned(),
        black: game.tags.get("Black").cloned(),
        result: game.result.clone().or_else(|| game.tags.get("Result").cloned()),
        plies: game.moves.len(),
        winner: color_from_result(game.result.as_ref()),
    }
}

fn average_if_nonzero(total: usize, count: usize) -> Option<f64> {
    if count == 0 {
        None
    } else {
        Some(total as f64 / count as f64)
    }
}

fn initial_position_for_record(record: &GameRecord) -> crate::board::Position {
    let fen = record
        .game
        .tags
        .get("FEN")
        .map(String::as_str)
        .unwrap_or(OFFICIAL_STARTPOS_FEN);
    parse_fen(fen).expect("record already reconstructed from valid initial FEN")
}

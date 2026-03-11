use std::collections::BTreeMap;

use serde::Serialize;

use crate::board::Color;
use crate::stats::{AggregateStats, GameSummary};

#[derive(Debug, Serialize)]
pub struct JsonGameSummary {
    pub event: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    pub plies: usize,
    pub winner: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JsonAggregateStats {
    pub games: usize,
    pub white_wins: usize,
    pub black_wins: usize,
    pub draws: usize,
    pub unresolved: usize,
    pub total_plies: usize,
    pub average_plies: f64,
    pub average_plies_white_wins: Option<f64>,
    pub average_plies_black_wins: Option<f64>,
    pub average_plies_draws: Option<f64>,
    pub average_plies_unresolved: Option<f64>,
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
}

pub fn to_json_summary(summary: &GameSummary) -> JsonGameSummary {
    JsonGameSummary {
        event: summary.event.clone(),
        white: summary.white.clone(),
        black: summary.black.clone(),
        result: summary.result.clone(),
        plies: summary.plies,
        winner: summary.winner.map(color_to_string),
    }
}

pub fn to_json_aggregate(stats: &AggregateStats) -> JsonAggregateStats {
    JsonAggregateStats {
        games: stats.games,
        white_wins: stats.white_wins,
        black_wins: stats.black_wins,
        draws: stats.draws,
        unresolved: stats.unresolved,
        total_plies: stats.total_plies,
        average_plies: stats.average_plies,
        average_plies_white_wins: stats.average_plies_white_wins,
        average_plies_black_wins: stats.average_plies_black_wins,
        average_plies_draws: stats.average_plies_draws,
        average_plies_unresolved: stats.average_plies_unresolved,
        white_first_moves: stats.white_first_moves.clone(),
        black_first_moves: stats.black_first_moves.clone(),
        games_with_kingside_castle: stats.games_with_kingside_castle,
        games_with_queenside_castle: stats.games_with_queenside_castle,
        games_with_no_castling: stats.games_with_no_castling,
        total_captures: stats.total_captures,
        average_captures: stats.average_captures,
        total_checks: stats.total_checks,
        average_checks: stats.average_checks,
        total_promotions: stats.total_promotions,
        average_promotions: stats.average_promotions,
    }
}

fn color_to_string(color: Color) -> String {
    match color {
        Color::White => "white".to_string(),
        Color::Black => "black".to_string(),
    }
}

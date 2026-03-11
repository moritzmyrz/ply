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
    }
}

fn color_to_string(color: Color) -> String {
    match color {
        Color::White => "white".to_string(),
        Color::Black => "black".to_string(),
    }
}

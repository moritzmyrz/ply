use std::collections::BTreeMap;

use crate::board::Color;
use crate::fen::parse_fen;
use crate::movegen::{apply_move, is_in_check};
use crate::opening::classify_opening;
use crate::pgn::{color_from_result, GameRecord, PgnGame, OFFICIAL_STARTPOS_FEN};

#[derive(Debug, Clone)]
pub struct GameSummary {
    pub event: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    pub plies: usize,
    pub winner: Option<Color>,
    pub eco: Option<String>,
    pub opening: Option<String>,
    pub variation: Option<String>,
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
    pub opening_frequencies: BTreeMap<String, usize>,
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

#[derive(Debug, Default)]
pub struct AggregateStatsAccumulator {
    stats: AggregateStats,
    white_wins_plies: usize,
    black_wins_plies: usize,
    draws_plies: usize,
    unresolved_plies: usize,
}

pub fn summarize_game(record: &GameRecord) -> GameSummary {
    let tags = &record.game.tags;
    let result = record.game.result.clone().or_else(|| tags.get("Result").cloned());
    let winner = color_from_result(result.as_ref());
    let opening = classify_opening(&record.plies);

    GameSummary {
        event: tags.get("Event").cloned(),
        white: tags.get("White").cloned(),
        black: tags.get("Black").cloned(),
        result,
        plies: record.plies.len(),
        winner,
        eco: opening.as_ref().map(|info| info.eco.to_string()),
        opening: opening.as_ref().map(|info| info.opening.to_string()),
        variation: opening.and_then(|info| info.variation.map(str::to_string)),
    }
}

pub fn summarize_games(records: &[GameRecord]) -> Vec<GameSummary> {
    records.iter().map(summarize_game).collect()
}

pub fn aggregate_stats(summaries: &[GameSummary]) -> AggregateStats {
    let mut acc = AggregateStatsAccumulator::default();
    for summary in summaries {
        acc.push_summary(summary);
    }
    acc.finish()
}

pub fn aggregate_record_stats(records: &[GameRecord]) -> AggregateStats {
    let mut acc = AggregateStatsAccumulator::default();
    for record in records {
        acc.push_record(record);
    }
    acc.finish()
}

pub fn summarize_raw_game(game: &PgnGame) -> GameSummary {
    GameSummary {
        event: game.tags.get("Event").cloned(),
        white: game.tags.get("White").cloned(),
        black: game.tags.get("Black").cloned(),
        result: game.result.clone().or_else(|| game.tags.get("Result").cloned()),
        plies: game.moves.len(),
        winner: color_from_result(game.result.as_ref()),
        eco: None,
        opening: None,
        variation: None,
    }
}

impl AggregateStatsAccumulator {
    pub fn push_summary(&mut self, summary: &GameSummary) {
        self.stats.games += 1;
        self.stats.total_plies += summary.plies;
        if let Some(opening) = summary.opening.as_ref() {
            *self.stats.opening_frequencies.entry(opening.clone()).or_insert(0) += 1;
        }

        match summary.result.as_deref() {
            Some("1-0") => {
                self.stats.white_wins += 1;
                self.white_wins_plies += summary.plies;
            }
            Some("0-1") => {
                self.stats.black_wins += 1;
                self.black_wins_plies += summary.plies;
            }
            Some("1/2-1/2") => {
                self.stats.draws += 1;
                self.draws_plies += summary.plies;
            }
            _ => {
                self.stats.unresolved += 1;
                self.unresolved_plies += summary.plies;
            }
        }
    }

    pub fn push_record(&mut self, record: &GameRecord) {
        let summary = summarize_game(record);
        self.push_summary(&summary);

        if let Some(first_white) = record.game.moves.first() {
            *self.stats.white_first_moves.entry(first_white.clone()).or_insert(0) += 1;
        }
        if let Some(first_black) = record.game.moves.get(1) {
            *self.stats.black_first_moves.entry(first_black.clone()).or_insert(0) += 1;
        }

        let mut saw_kingside = false;
        let mut saw_queenside = false;
        let mut position = initial_position_for_record(record);

        for mv in &record.plies {
            if mv.is_capture {
                self.stats.total_captures += 1;
            }
            if mv.promotion.is_some() {
                self.stats.total_promotions += 1;
            }
            match mv.castle {
                Some(crate::board::CastleSide::KingSide) => saw_kingside = true,
                Some(crate::board::CastleSide::QueenSide) => saw_queenside = true,
                None => {}
            }

            apply_move(&mut position, *mv);
            if is_in_check(&position, position.side_to_move) {
                self.stats.total_checks += 1;
            }
        }

        if saw_kingside {
            self.stats.games_with_kingside_castle += 1;
        }
        if saw_queenside {
            self.stats.games_with_queenside_castle += 1;
        }
        if !saw_kingside && !saw_queenside {
            self.stats.games_with_no_castling += 1;
        }
    }

    pub fn finish(mut self) -> AggregateStats {
        if self.stats.games > 0 {
            let games = self.stats.games as f64;
            self.stats.average_plies = self.stats.total_plies as f64 / games;
            self.stats.average_captures = self.stats.total_captures as f64 / games;
            self.stats.average_checks = self.stats.total_checks as f64 / games;
            self.stats.average_promotions = self.stats.total_promotions as f64 / games;
        }
        self.stats.average_plies_white_wins =
            average_if_nonzero(self.white_wins_plies, self.stats.white_wins);
        self.stats.average_plies_black_wins =
            average_if_nonzero(self.black_wins_plies, self.stats.black_wins);
        self.stats.average_plies_draws = average_if_nonzero(self.draws_plies, self.stats.draws);
        self.stats.average_plies_unresolved =
            average_if_nonzero(self.unresolved_plies, self.stats.unresolved);
        self.stats
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

use crate::board::Color;
use crate::pgn::{color_from_result, GameRecord, PgnGame};

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
    for summary in summaries {
        stats.total_plies += summary.plies;
        match summary.result.as_deref() {
            Some("1-0") => stats.white_wins += 1,
            Some("0-1") => stats.black_wins += 1,
            Some("1/2-1/2") => stats.draws += 1,
            _ => stats.unresolved += 1,
        }
    }
    if stats.games > 0 {
        stats.average_plies = stats.total_plies as f64 / stats.games as f64;
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

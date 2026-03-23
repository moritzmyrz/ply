use ply::export::{to_json_aggregate, to_json_summary};
use ply::stats::{AggregateStats, GameSummary};

use super::CliError;

#[derive(Debug, Clone)]
pub struct ValidateFailure {
    pub game_index: usize,
    pub message: String,
}

pub struct ValidateOutput {
    pub validated_games: usize,
    pub valid: usize,
    pub invalid: usize,
    pub failures: Vec<ValidateFailure>,
}

pub enum SummaryEntry {
    Valid { index: usize, summary: GameSummary },
    Invalid { index: usize, error: String },
}

pub struct SummariesOutput {
    pub entries: Vec<SummaryEntry>,
}

pub struct StatsOutput {
    pub json: bool,
    pub stats: AggregateStats,
    pub summaries: Vec<GameSummary>,
}

pub struct FenOutput {
    pub normalized_fen: String,
    pub legal_moves: Vec<String>,
}

pub struct PerftOutput {
    pub depth: u8,
    pub nodes: u64,
    pub elapsed_ms: u128,
    pub nps: u64,
    pub divide: Option<Vec<(String, u64)>>,
}

pub fn render_validate(output: &ValidateOutput) -> String {
    let mut rendered = format!(
        "validated games: {}\nvalid: {}\ninvalid: {}\n",
        output.validated_games, output.valid, output.invalid
    );
    if !output.failures.is_empty() {
        rendered.push_str("\nfailures:\n");
        for f in &output.failures {
            rendered.push_str(&format!("  game {}: {}\n", f.game_index, f.message));
        }
    }
    rendered
}

pub fn render_summaries(output: &SummariesOutput) -> String {
    let mut rendered = String::new();
    for entry in &output.entries {
        match entry {
            SummaryEntry::Valid { index, summary } => {
                rendered.push_str(&format!(
                    "{}. {} vs {} | result={} | plies={}",
                    index,
                    summary.white.as_deref().unwrap_or("?"),
                    summary.black.as_deref().unwrap_or("?"),
                    summary.result.as_deref().unwrap_or("*"),
                    summary.plies
                ));
                if let Some(opening) = summary.opening.as_deref() {
                    rendered.push_str(&format!(" | opening={opening}"));
                }
                if let Some(eco) = summary.eco.as_deref() {
                    rendered.push_str(&format!(" | eco={eco}"));
                }
                rendered.push('\n');
            }
            SummaryEntry::Invalid { index, error } => {
                rendered.push_str(&format!("{index}. invalid game: {error}\n"));
            }
        }
    }
    rendered
}

pub fn render_stats(output: &StatsOutput) -> Result<String, CliError> {
    if output.json {
        let payload = serde_json::json!({
            "stats": to_json_aggregate(&output.stats),
            "games": output.summaries.iter().map(to_json_summary).collect::<Vec<_>>(),
        });
        return Ok(format!("{}\n", serde_json::to_string_pretty(&payload)?));
    }

    let mut rendered = String::new();
    rendered.push_str(&format!("games: {}\n", output.stats.games));
    rendered.push_str("results:\n");
    rendered.push_str(&format!("  white_wins: {}\n", output.stats.white_wins));
    rendered.push_str(&format!("  black_wins: {}\n", output.stats.black_wins));
    rendered.push_str(&format!("  draws: {}\n", output.stats.draws));
    rendered.push_str(&format!("  unresolved: {}\n", output.stats.unresolved));

    rendered.push_str("length:\n");
    rendered.push_str(&format!("  total_plies: {}\n", output.stats.total_plies));
    rendered.push_str(&format!("  average_plies: {:.2}\n", output.stats.average_plies));
    rendered.push_str(&format!(
        "  average_plies_white_wins: {}\n",
        format_optional(output.stats.average_plies_white_wins)
    ));
    rendered.push_str(&format!(
        "  average_plies_black_wins: {}\n",
        format_optional(output.stats.average_plies_black_wins)
    ));
    rendered.push_str(&format!(
        "  average_plies_draws: {}\n",
        format_optional(output.stats.average_plies_draws)
    ));
    rendered.push_str(&format!(
        "  average_plies_unresolved: {}\n",
        format_optional(output.stats.average_plies_unresolved)
    ));

    rendered.push_str("first_moves_white:\n");
    for (mv, count) in &output.stats.white_first_moves {
        rendered.push_str(&format!("  {mv}: {count}\n"));
    }
    rendered.push_str("first_moves_black:\n");
    for (mv, count) in &output.stats.black_first_moves {
        rendered.push_str(&format!("  {mv}: {count}\n"));
    }
    rendered.push_str("openings:\n");
    for (opening, count) in &output.stats.opening_frequencies {
        rendered.push_str(&format!("  {opening}: {count}\n"));
    }

    rendered.push_str("castling:\n");
    rendered.push_str(&format!(
        "  games_with_kingside_castle: {}\n",
        output.stats.games_with_kingside_castle
    ));
    rendered.push_str(&format!(
        "  games_with_queenside_castle: {}\n",
        output.stats.games_with_queenside_castle
    ));
    rendered
        .push_str(&format!("  games_with_no_castling: {}\n", output.stats.games_with_no_castling));

    rendered.push_str("move_events:\n");
    rendered.push_str(&format!("  total_captures: {}\n", output.stats.total_captures));
    rendered.push_str(&format!("  average_captures: {:.2}\n", output.stats.average_captures));
    rendered.push_str(&format!("  total_checks: {}\n", output.stats.total_checks));
    rendered.push_str(&format!("  average_checks: {:.2}\n", output.stats.average_checks));
    rendered.push_str(&format!("  total_promotions: {}\n", output.stats.total_promotions));
    rendered.push_str(&format!("  average_promotions: {:.2}\n", output.stats.average_promotions));
    Ok(rendered)
}

pub fn render_fen(output: &FenOutput) -> String {
    let mut rendered = format!("normalized_fen: {}\n", output.normalized_fen);
    if !output.legal_moves.is_empty() {
        rendered.push_str(&format!(
            "legal_moves ({}): {}\n",
            output.legal_moves.len(),
            output.legal_moves.join(" ")
        ));
    }
    rendered
}

pub fn render_perft(output: &PerftOutput) -> String {
    let mut rendered = format!(
        "depth: {}\nnodes: {}\nelapsed_ms: {}\nnps: {}\n",
        output.depth, output.nodes, output.elapsed_ms, output.nps
    );
    if let Some(divide) = output.divide.as_ref() {
        for (mv, count) in divide {
            rendered.push_str(&format!("{mv}: {count}\n"));
        }
    }
    rendered
}

fn format_optional(value: Option<f64>) -> String {
    value.map(|v| format!("{v:.2}")).unwrap_or_else(|| "-".to_string())
}

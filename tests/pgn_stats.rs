use std::fs;

use ply::pgn::{parse_pgn, reconstruct_game};
use ply::stats::{aggregate_record_stats, summarize_games};

#[test]
fn parse_and_reconstruct_fixture_games() {
    let input =
        fs::read_to_string("tests/fixtures/sample_games.pgn").expect("fixture should be readable");
    let games = parse_pgn(&input).expect("valid pgn");
    assert_eq!(games.len(), 2);

    let records = games
        .iter()
        .map(reconstruct_game)
        .collect::<Result<Vec<_>, _>>()
        .expect("games should reconstruct");
    assert_eq!(records[0].plies.len(), 7);
    assert_eq!(records[1].plies.len(), 8);
}

#[test]
fn aggregate_stats_from_fixture_games() {
    let input =
        fs::read_to_string("tests/fixtures/sample_games.pgn").expect("fixture should be readable");
    let games = parse_pgn(&input).expect("valid pgn");
    let records = games
        .iter()
        .map(reconstruct_game)
        .collect::<Result<Vec<_>, _>>()
        .expect("games should reconstruct");
    let summaries = summarize_games(&records);
    let stats = aggregate_record_stats(&records);

    assert_eq!(stats.games, 2);
    assert_eq!(stats.white_wins, 1);
    assert_eq!(stats.draws, 1);
    assert_eq!(stats.total_plies, 15);
    assert_eq!(stats.average_plies, 7.5);
    assert_eq!(stats.white_first_moves.get("e4"), Some(&1));
    assert_eq!(stats.white_first_moves.get("d4"), Some(&1));
    assert_eq!(stats.black_first_moves.get("e5"), Some(&1));
    assert_eq!(stats.black_first_moves.get("d5"), Some(&1));
    assert_eq!(stats.games_with_kingside_castle, 0);
    assert_eq!(stats.games_with_queenside_castle, 0);
    assert_eq!(stats.games_with_no_castling, 2);
    assert_eq!(stats.total_captures, 1);
    assert_eq!(stats.average_captures, 0.5);
    assert_eq!(stats.total_checks, 1);
    assert_eq!(stats.average_checks, 0.5);
    assert_eq!(stats.total_promotions, 0);
    assert_eq!(stats.average_promotions, 0.0);
    assert_eq!(stats.average_plies_white_wins, Some(7.0));
    assert_eq!(stats.average_plies_draws, Some(8.0));
    assert_eq!(stats.average_plies_black_wins, None);
    assert_eq!(stats.average_plies_unresolved, None);

    assert_eq!(summaries.len(), 2);
}

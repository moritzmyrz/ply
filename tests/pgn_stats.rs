use std::fs;

use ply::pgn::{parse_pgn, reconstruct_game};
use ply::stats::{aggregate_stats, summarize_games};

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
    let stats = aggregate_stats(&summaries);

    assert_eq!(stats.games, 2);
    assert_eq!(stats.white_wins, 1);
    assert_eq!(stats.draws, 1);
}

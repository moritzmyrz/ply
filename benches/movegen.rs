use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ply::fen::{parse_fen, STARTPOS_FEN};
use ply::movegen::generate_legal_moves;
use ply::perft::perft;
use ply::pgn::{parse_pgn, reconstruct_game};

fn bench_startpos_movegen(c: &mut Criterion) {
    let pos = parse_fen(STARTPOS_FEN).expect("valid start position");
    c.bench_function("startpos_legal_movegen", |b| {
        b.iter(|| {
            let moves = generate_legal_moves(black_box(&pos));
            black_box(moves.len());
        })
    });
}

fn bench_perft(c: &mut Criterion) {
    let pos = parse_fen(STARTPOS_FEN).expect("valid start position");
    c.bench_function("startpos_perft_depth3", |b| b.iter(|| black_box(perft(&pos, 3))));
    c.bench_function("startpos_perft_depth4", |b| b.iter(|| black_box(perft(&pos, 4))));
}

fn bench_reconstruct_fixture(c: &mut Criterion) {
    let input = fs::read_to_string("tests/fixtures/sample_games.pgn").expect("fixture should load");
    let games = parse_pgn(&input).expect("fixture should parse");
    c.bench_function("sample_pgn_reconstruct", |b| {
        b.iter(|| {
            for game in &games {
                black_box(reconstruct_game(game).expect("fixture should reconstruct"));
            }
        })
    });
}

criterion_group!(benches, bench_startpos_movegen, bench_perft, bench_reconstruct_fixture);
criterion_main!(benches);

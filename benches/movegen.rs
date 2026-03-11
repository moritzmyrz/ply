use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ply::fen::{parse_fen, STARTPOS_FEN};
use ply::movegen::generate_legal_moves;

fn bench_startpos_movegen(c: &mut Criterion) {
    let pos = parse_fen(STARTPOS_FEN).expect("valid start position");
    c.bench_function("startpos_legal_movegen", |b| {
        b.iter(|| {
            let moves = generate_legal_moves(black_box(&pos));
            black_box(moves.len());
        })
    });
}

criterion_group!(benches, bench_startpos_movegen);
criterion_main!(benches);

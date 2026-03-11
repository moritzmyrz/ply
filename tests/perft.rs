use ply::fen::{parse_fen, STARTPOS_FEN};
use ply::perft::perft;

#[test]
fn perft_start_position_matches_known_counts() {
    let pos = parse_fen(STARTPOS_FEN).expect("start position should parse");
    assert_eq!(perft(&pos, 1), 20);
    assert_eq!(perft(&pos, 2), 400);
    assert_eq!(perft(&pos, 3), 8_902);
}

#[test]
fn perft_kiwipete_position_matches_known_counts() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let pos = parse_fen(fen).expect("kiwipete should parse");
    assert_eq!(perft(&pos, 1), 48);
    assert_eq!(perft(&pos, 2), 2_039);
}

#[test]
fn perft_en_passant_stress_position_matches_known_counts() {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let pos = parse_fen(fen).expect("position should parse");
    assert_eq!(perft(&pos, 1), 14);
    assert_eq!(perft(&pos, 2), 191);
}

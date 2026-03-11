use ply::fen::{parse_fen, to_fen, STARTPOS_FEN};
use ply::movegen::generate_legal_moves;

#[test]
fn fen_roundtrip_start_position() {
    let pos = parse_fen(STARTPOS_FEN).expect("parse");
    assert_eq!(to_fen(&pos), STARTPOS_FEN);
}

#[test]
fn start_position_has_20_legal_moves() {
    let pos = parse_fen(STARTPOS_FEN).expect("parse");
    let legal = generate_legal_moves(&pos);
    assert_eq!(legal.len(), 20);
}

#[test]
fn legal_movegen_handles_en_passant_target() {
    let fen = "rnbqkbnr/ppp2ppp/8/3pp3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
    let pos = parse_fen(fen).expect("parse");
    let legal = generate_legal_moves(&pos);
    assert!(!legal.is_empty());
}

use ply::fen::{parse_fen, to_fen, STARTPOS_FEN};
use ply::movegen::{apply_move_with_undo, generate_legal_moves, undo_move};
use ply::pgn::{parse_pgn, reconstruct_game};

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

#[test]
fn fen_tracks_king_squares() {
    let pos = parse_fen(STARTPOS_FEN).expect("parse");
    assert_eq!(pos.white_king.to_algebraic(), "e1");
    assert_eq!(pos.black_king.to_algebraic(), "e8");
}

#[test]
fn apply_and_undo_move_restores_position() {
    let mut pos = parse_fen(STARTPOS_FEN).expect("parse");
    let original = pos.clone();
    let mv = generate_legal_moves(&pos)
        .into_iter()
        .find(|mv| mv.to_coordinate() == "e2e4")
        .expect("e2e4 should be legal");
    let undo = apply_move_with_undo(&mut pos, mv);
    undo_move(&mut pos, mv, undo);
    assert_eq!(pos, original);
}

#[test]
fn opening_classification_is_exposed_in_summary() {
    let input = "[Event \"Opening\"]\n[White \"W\"]\n[Black \"B\"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0\n";
    let game = parse_pgn(input).expect("valid pgn").pop().expect("one game");
    let record = reconstruct_game(&game).expect("reconstruct");
    let summary = ply::stats::summarize_game(&record);
    assert_eq!(summary.opening.as_deref(), Some("Ruy Lopez"));
    assert_eq!(summary.eco.as_deref(), Some("C60"));
}

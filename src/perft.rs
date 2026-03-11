use crate::board::Position;
use crate::movegen::{apply_move, generate_legal_moves};

pub fn perft(position: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let legal_moves = generate_legal_moves(position);
    if depth == 1 {
        return legal_moves.len() as u64;
    }

    let mut nodes = 0u64;
    for mv in legal_moves {
        let mut next = position.clone();
        apply_move(&mut next, mv);
        nodes += perft(&next, depth - 1);
    }
    nodes
}

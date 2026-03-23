use crate::board::{ChessMove, Position};
use crate::movegen::{apply_move_with_undo, generate_legal_moves_into, undo_move};

pub fn perft(position: &Position, depth: u8) -> u64 {
    let mut position = position.clone();
    let mut buffers = vec![Vec::new(); depth as usize + 1];
    perft_inner(&mut position, depth, &mut buffers, 0)
}

pub fn perft_divide(position: &Position, depth: u8) -> Vec<(ChessMove, u64)> {
    let mut position = position.clone();
    let mut buffers = vec![Vec::new(); depth as usize + 1];
    let mut moves = std::mem::take(&mut buffers[0]);
    generate_legal_moves_into(&position, &mut moves);

    let mut divided = Vec::with_capacity(moves.len());
    if depth == 0 {
        buffers[0] = moves;
        return divided;
    }

    for &mv in &moves {
        let nodes = if depth == 1 {
            1
        } else {
            let undo = apply_move_with_undo(&mut position, mv);
            let nodes = perft_inner(&mut position, depth - 1, &mut buffers, 1);
            undo_move(&mut position, mv, undo);
            nodes
        };
        divided.push((mv, nodes));
    }

    buffers[0] = moves;
    divided
}

fn perft_inner(
    position: &mut Position,
    depth: u8,
    buffers: &mut [Vec<ChessMove>],
    ply: usize,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = std::mem::take(&mut buffers[ply]);
    generate_legal_moves_into(position, &mut moves);
    if depth == 1 {
        let count = moves.len() as u64;
        buffers[ply] = moves;
        return count;
    }

    let mut nodes = 0u64;
    for &mv in &moves {
        let undo = apply_move_with_undo(position, mv);
        nodes += perft_inner(position, depth - 1, buffers, ply + 1);
        undo_move(position, mv, undo);
    }

    buffers[ply] = moves;
    nodes
}

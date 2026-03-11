use crate::board::{
    CastleSide, ChessMove, Color, Piece, PieceKind, Position, Square,
};

pub fn generate_legal_moves(position: &Position) -> Vec<ChessMove> {
    let pseudo = generate_pseudo_legal_moves(position);
    pseudo
        .into_iter()
        .filter(|mv| {
            let mut next = position.clone();
            apply_move(&mut next, *mv);
            !is_in_check(&next, position.side_to_move)
        })
        .collect()
}

pub fn is_in_check(position: &Position, color: Color) -> bool {
    let king_sq = match position.king_square(color) {
        Some(sq) => sq,
        None => return false,
    };
    is_square_attacked(position, king_sq, color.opposite())
}

pub fn is_square_attacked(position: &Position, target: Square, by: Color) -> bool {
    for idx in 0..64 {
        let from = Square(idx as u8);
        let Some(piece) = position.piece_at(from) else {
            continue;
        };
        if piece.color != by {
            continue;
        }
        let deltas: &[(i8, i8)] = match piece.kind {
            PieceKind::Pawn => {
                if by == Color::White {
                    &[(-1, 1), (1, 1)]
                } else {
                    &[(-1, -1), (1, -1)]
                }
            }
            PieceKind::Knight => &[
                (-2, -1),
                (-2, 1),
                (-1, -2),
                (-1, 2),
                (1, -2),
                (1, 2),
                (2, -1),
                (2, 1),
            ],
            PieceKind::Bishop => &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
            PieceKind::Rook => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
            PieceKind::Queen => &[
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
            ],
            PieceKind::King => &[
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
            ],
        };

        match piece.kind {
            PieceKind::Pawn | PieceKind::Knight | PieceKind::King => {
                for (df, dr) in deltas {
                    if let Some(to) = offset_square(from, *df, *dr) {
                        if to == target {
                            return true;
                        }
                    }
                }
            }
            PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen => {
                for (df, dr) in deltas {
                    let mut cur = from;
                    while let Some(next) = offset_square(cur, *df, *dr) {
                        cur = next;
                        if cur == target {
                            return true;
                        }
                        if position.piece_at(cur).is_some() {
                            break;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn apply_move(position: &mut Position, mv: ChessMove) {
    let piece = position.piece_at(mv.from);
    position.set_piece(mv.from, None);
    if mv.is_en_passant {
        if let Some(p) = piece {
            let capture_rank = if p.color == Color::White {
                mv.to.rank() - 1
            } else {
                mv.to.rank() + 1
            };
            let capture_sq = Square::from_coords(mv.to.file(), capture_rank).expect("capture square");
            position.set_piece(capture_sq, None);
        }
    }
    if let Some(side) = mv.castle {
        apply_castle_rook_move(position, side, piece.expect("king exists").color);
    }
    let moved_piece = match (piece, mv.promotion) {
        (Some(mut p), Some(prom)) => {
            p.kind = prom;
            Some(p)
        }
        (None, Some(_)) => None,
        (p, None) => p,
    };
    position.set_piece(mv.to, moved_piece);
    update_castling_after_move(position, piece, mv);
    update_en_passant_target(position, piece, mv);
    update_move_counters(position, piece, mv);
    position.side_to_move = position.side_to_move.opposite();
}

pub fn generate_pseudo_legal_moves(position: &Position) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    for idx in 0..64 {
        let from = Square(idx as u8);
        let Some(piece) = position.piece_at(from) else {
            continue;
        };
        if piece.color != position.side_to_move {
            continue;
        }
        match piece.kind {
            PieceKind::Pawn => push_pawn_moves(position, from, piece, &mut moves),
            PieceKind::Knight => push_knight_moves(position, from, piece, &mut moves),
            PieceKind::Bishop => push_slider_moves(
                position,
                from,
                piece,
                &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
                &mut moves,
            ),
            PieceKind::Rook => push_slider_moves(
                position,
                from,
                piece,
                &[(-1, 0), (1, 0), (0, -1), (0, 1)],
                &mut moves,
            ),
            PieceKind::Queen => push_slider_moves(
                position,
                from,
                piece,
                &[
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                ],
                &mut moves,
            ),
            PieceKind::King => {
                push_king_moves(position, from, piece, &mut moves);
                push_castling_moves(position, from, piece, &mut moves);
            }
        }
    }
    moves
}

fn push_pawn_moves(position: &Position, from: Square, piece: Piece, out: &mut Vec<ChessMove>) {
    let dir = if piece.color == Color::White { 1 } else { -1 };
    let start_rank = if piece.color == Color::White { 1 } else { 6 };
    let promo_rank = if piece.color == Color::White { 7 } else { 0 };

    if let Some(one) = offset_square(from, 0, dir) {
        if position.piece_at(one).is_none() {
            push_pawn_advance(from, one, promo_rank, out);
            if from.rank() == start_rank {
                if let Some(two) = offset_square(from, 0, dir * 2) {
                    if position.piece_at(two).is_none() {
                        out.push(ChessMove::new(from, two));
                    }
                }
            }
        }
    }

    for df in [-1, 1] {
        if let Some(to) = offset_square(from, df, dir) {
            let target_piece = position.piece_at(to);
            if let Some(tp) = target_piece {
                if tp.color != piece.color {
                    if to.rank() == promo_rank {
                        for promo in [
                            PieceKind::Queen,
                            PieceKind::Rook,
                            PieceKind::Bishop,
                            PieceKind::Knight,
                        ] {
                            let mut mv = ChessMove::new(from, to);
                            mv.is_capture = true;
                            mv.promotion = Some(promo);
                            out.push(mv);
                        }
                    } else {
                        let mut mv = ChessMove::new(from, to);
                        mv.is_capture = true;
                        out.push(mv);
                    }
                }
            } else if position.en_passant_target == Some(to) {
                let mut mv = ChessMove::new(from, to);
                mv.is_capture = true;
                mv.is_en_passant = true;
                out.push(mv);
            }
        }
    }
}

fn push_pawn_advance(from: Square, to: Square, promo_rank: u8, out: &mut Vec<ChessMove>) {
    if to.rank() == promo_rank {
        for promo in [
            PieceKind::Queen,
            PieceKind::Rook,
            PieceKind::Bishop,
            PieceKind::Knight,
        ] {
            let mut mv = ChessMove::new(from, to);
            mv.promotion = Some(promo);
            out.push(mv);
        }
    } else {
        out.push(ChessMove::new(from, to));
    }
}

fn push_knight_moves(position: &Position, from: Square, piece: Piece, out: &mut Vec<ChessMove>) {
    for (df, dr) in [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ] {
        if let Some(to) = offset_square(from, df, dr) {
            match position.piece_at(to) {
                None => out.push(ChessMove::new(from, to)),
                Some(p) if p.color != piece.color => {
                    let mut mv = ChessMove::new(from, to);
                    mv.is_capture = true;
                    out.push(mv);
                }
                _ => {}
            }
        }
    }
}

fn push_slider_moves(
    position: &Position,
    from: Square,
    piece: Piece,
    deltas: &[(i8, i8)],
    out: &mut Vec<ChessMove>,
) {
    for (df, dr) in deltas {
        let mut cur = from;
        while let Some(to) = offset_square(cur, *df, *dr) {
            cur = to;
            match position.piece_at(to) {
                None => out.push(ChessMove::new(from, to)),
                Some(p) if p.color != piece.color => {
                    let mut mv = ChessMove::new(from, to);
                    mv.is_capture = true;
                    out.push(mv);
                    break;
                }
                _ => break,
            }
        }
    }
}

fn push_king_moves(position: &Position, from: Square, piece: Piece, out: &mut Vec<ChessMove>) {
    for (df, dr) in [
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
    ] {
        if let Some(to) = offset_square(from, df, dr) {
            match position.piece_at(to) {
                None => out.push(ChessMove::new(from, to)),
                Some(p) if p.color != piece.color => {
                    let mut mv = ChessMove::new(from, to);
                    mv.is_capture = true;
                    out.push(mv);
                }
                _ => {}
            }
        }
    }
}

fn push_castling_moves(position: &Position, from: Square, piece: Piece, out: &mut Vec<ChessMove>) {
    if piece.kind != PieceKind::King {
        return;
    }
    if is_in_check(position, piece.color) {
        return;
    }
    let (rank, king_side_ok, queen_side_ok) = match piece.color {
        Color::White => (
            0u8,
            position.castling.white_king_side,
            position.castling.white_queen_side,
        ),
        Color::Black => (
            7u8,
            position.castling.black_king_side,
            position.castling.black_queen_side,
        ),
    };

    if from != Square::from_coords(4, rank).expect("e-file") {
        return;
    }

    if king_side_ok {
        let f = Square::from_coords(5, rank).expect("f");
        let g = Square::from_coords(6, rank).expect("g");
        if position.piece_at(f).is_none()
            && position.piece_at(g).is_none()
            && !is_square_attacked(position, f, piece.color.opposite())
            && !is_square_attacked(position, g, piece.color.opposite())
        {
            let mut mv = ChessMove::new(from, g);
            mv.castle = Some(CastleSide::KingSide);
            out.push(mv);
        }
    }

    if queen_side_ok {
        let b = Square::from_coords(1, rank).expect("b");
        let c = Square::from_coords(2, rank).expect("c");
        let d = Square::from_coords(3, rank).expect("d");
        if position.piece_at(b).is_none()
            && position.piece_at(c).is_none()
            && position.piece_at(d).is_none()
            && !is_square_attacked(position, c, piece.color.opposite())
            && !is_square_attacked(position, d, piece.color.opposite())
        {
            let mut mv = ChessMove::new(from, c);
            mv.castle = Some(CastleSide::QueenSide);
            out.push(mv);
        }
    }
}

fn apply_castle_rook_move(position: &mut Position, side: CastleSide, color: Color) {
    let rank = if color == Color::White { 0 } else { 7 };
    let (rook_from, rook_to) = match side {
        CastleSide::KingSide => (
            Square::from_coords(7, rank).expect("rook"),
            Square::from_coords(5, rank).expect("rook"),
        ),
        CastleSide::QueenSide => (
            Square::from_coords(0, rank).expect("rook"),
            Square::from_coords(3, rank).expect("rook"),
        ),
    };
    let rook = position.piece_at(rook_from);
    position.set_piece(rook_from, None);
    position.set_piece(rook_to, rook);
}

fn update_castling_after_move(position: &mut Position, moved_piece: Option<Piece>, mv: ChessMove) {
    if let Some(piece) = moved_piece {
        match piece.kind {
            PieceKind::King => match piece.color {
                Color::White => {
                    position.castling.white_king_side = false;
                    position.castling.white_queen_side = false;
                }
                Color::Black => {
                    position.castling.black_king_side = false;
                    position.castling.black_queen_side = false;
                }
            },
            PieceKind::Rook => {
                disable_rook_castle(position, mv.from);
            }
            _ => {}
        }
    }
    disable_rook_castle(position, mv.to);
}

fn disable_rook_castle(position: &mut Position, sq: Square) {
    match (sq.file(), sq.rank()) {
        (0, 0) => position.castling.white_queen_side = false,
        (7, 0) => position.castling.white_king_side = false,
        (0, 7) => position.castling.black_queen_side = false,
        (7, 7) => position.castling.black_king_side = false,
        _ => {}
    }
}

fn update_en_passant_target(position: &mut Position, moved_piece: Option<Piece>, mv: ChessMove) {
    position.en_passant_target = None;
    if let Some(piece) = moved_piece {
        if piece.kind == PieceKind::Pawn {
            let from_rank = mv.from.rank() as i8;
            let to_rank = mv.to.rank() as i8;
            if (from_rank - to_rank).abs() == 2 {
                let mid_rank = ((from_rank + to_rank) / 2) as u8;
                position.en_passant_target =
                    Square::from_coords(mv.from.file(), mid_rank);
            }
        }
    }
}

fn update_move_counters(position: &mut Position, moved_piece: Option<Piece>, mv: ChessMove) {
    if mv.is_capture || moved_piece.map(|p| p.kind == PieceKind::Pawn).unwrap_or(false) {
        position.halfmove_clock = 0;
    } else {
        position.halfmove_clock += 1;
    }
    if position.side_to_move == Color::Black {
        position.fullmove_number += 1;
    }
}

fn offset_square(from: Square, df: i8, dr: i8) -> Option<Square> {
    let nf = from.file() as i8 + df;
    let nr = from.rank() as i8 + dr;
    if (0..8).contains(&nf) && (0..8).contains(&nr) {
        Square::from_coords(nf as u8, nr as u8)
    } else {
        None
    }
}

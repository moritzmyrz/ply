use crate::board::{CastleSide, ChessMove, PieceKind, Position, Square};
use crate::movegen::{apply_move, generate_legal_moves, is_in_check};

pub fn resolve_san(position: &Position, legal: &[ChessMove], san: &str) -> Option<ChessMove> {
    let clean = normalize_san(san);
    if clean == "O-O" {
        return legal.iter().copied().find(|m| m.castle == Some(CastleSide::KingSide));
    }
    if clean == "O-O-O" {
        return legal.iter().copied().find(|m| m.castle == Some(CastleSide::QueenSide));
    }

    let target = parse_target_square(&clean)?;
    let promo = parse_promotion(&clean);
    let piece_kind = parse_piece_kind(&clean);
    let capture = clean.contains('x');
    let (from_file_hint, from_rank_hint) = parse_disambiguation(&clean, piece_kind);

    let mut candidates: Vec<ChessMove> = legal
        .iter()
        .copied()
        .filter(|mv| mv.to == target)
        .filter(|mv| mv.promotion == promo)
        .filter(|mv| mv.is_capture == capture || (piece_kind == PieceKind::Pawn && capture))
        .filter(|mv| position.piece_at(mv.from).map(|p| p.kind == piece_kind).unwrap_or(false))
        .collect();

    if let Some(file) = from_file_hint {
        candidates.retain(|mv| mv.from.file() == file);
    }
    if let Some(rank) = from_rank_hint {
        candidates.retain(|mv| mv.from.rank() == rank);
    }
    candidates.into_iter().next()
}

pub fn move_to_san(position: &Position, mv: ChessMove) -> String {
    if mv.castle == Some(CastleSide::KingSide) {
        return "O-O".to_string();
    }
    if mv.castle == Some(CastleSide::QueenSide) {
        return "O-O-O".to_string();
    }
    let piece = position.piece_at(mv.from).expect("piece should exist");
    let mut out = String::new();
    if let Some(c) = piece.kind.san_char() {
        out.push(c);
    } else if mv.is_capture {
        out.push((b'a' + mv.from.file()) as char);
    }
    if mv.is_capture {
        out.push('x');
    }
    out.push_str(&mv.to.to_algebraic());
    if let Some(prom) = mv.promotion.and_then(|p| p.san_char()) {
        out.push('=');
        out.push(prom);
    }
    let mut next = position.clone();
    apply_move(&mut next, mv);
    if is_in_check(&next, next.side_to_move) {
        let replies = generate_legal_moves(&next);
        if replies.is_empty() {
            out.push('#');
        } else {
            out.push('+');
        }
    }
    out
}

fn normalize_san(san: &str) -> String {
    san.trim()
        .trim_end_matches('+')
        .trim_end_matches('#')
        .trim_end_matches('!')
        .trim_end_matches('?')
        .replace("0-0-0", "O-O-O")
        .replace("0-0", "O-O")
}

fn parse_piece_kind(san: &str) -> PieceKind {
    let first = san.chars().next().unwrap_or(' ');
    match first {
        'N' => PieceKind::Knight,
        'B' => PieceKind::Bishop,
        'R' => PieceKind::Rook,
        'Q' => PieceKind::Queen,
        'K' => PieceKind::King,
        _ => PieceKind::Pawn,
    }
}

fn parse_target_square(san: &str) -> Option<Square> {
    let bytes = san.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    for i in (1..bytes.len()).rev() {
        let file = bytes[i - 1];
        let rank = bytes[i];
        if (b'a'..=b'h').contains(&file) && (b'1'..=b'8').contains(&rank) {
            let sq = [file as char, rank as char].iter().collect::<String>();
            return Square::from_algebraic(&sq);
        }
    }
    None
}

fn parse_promotion(san: &str) -> Option<PieceKind> {
    let marker = san.find('=').or_else(|| san.find('/'))?;
    let c = san.chars().nth(marker + 1)?;
    match c {
        'N' => Some(PieceKind::Knight),
        'B' => Some(PieceKind::Bishop),
        'R' => Some(PieceKind::Rook),
        'Q' => Some(PieceKind::Queen),
        _ => None,
    }
}

fn parse_disambiguation(san: &str, piece_kind: PieceKind) -> (Option<u8>, Option<u8>) {
    let mut span = san.to_string();
    if let Some(idx) = span.find('=') {
        span.truncate(idx);
    }
    while span.ends_with('+') || span.ends_with('#') || span.ends_with('!') || span.ends_with('?') {
        span.pop();
    }
    if span.len() < 2 {
        return (None, None);
    }

    let dest_start = span
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i.saturating_sub(1))
        .unwrap_or(0);

    let mut prefix = span[..dest_start].to_string();
    prefix = prefix.replace('x', "");
    if piece_kind != PieceKind::Pawn && !prefix.is_empty() {
        prefix.remove(0);
    }
    let mut file_hint = None;
    let mut rank_hint = None;
    for c in prefix.chars() {
        if ('a'..='h').contains(&c) {
            file_hint = Some(c as u8 - b'a');
        } else if ('1'..='8').contains(&c) {
            rank_hint = Some(c as u8 - b'1');
        }
    }
    (file_hint, rank_hint)
}

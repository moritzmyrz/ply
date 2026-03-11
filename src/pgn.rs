use std::collections::BTreeMap;

use crate::board::{CastleSide, ChessMove, Color, PieceKind, Position, Square};
use crate::fen::parse_fen;
use crate::movegen::{apply_move, generate_legal_moves, is_in_check};

pub const OFFICIAL_STARTPOS_FEN: &str =
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone)]
pub struct PgnGame {
    pub tags: BTreeMap<String, String>,
    pub moves: Vec<String>,
    pub result: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GameRecord {
    pub game: PgnGame,
    pub position: Position,
    pub plies: Vec<ChessMove>,
}

#[derive(Debug, Clone)]
pub struct ReplayError {
    pub ply_index: usize,
    pub san: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum PgnError {
    Parse(String),
    InvalidInitialFen(String),
    Replay(ReplayError),
}

pub fn parse_pgn(input: &str) -> Result<Vec<PgnGame>, PgnError> {
    let mut games = Vec::new();
    let chunks = split_games(input);
    for chunk in chunks {
        let game = parse_single_game(&chunk)?;
        if !game.tags.is_empty() || !game.moves.is_empty() || game.result.is_some() {
            games.push(game);
        }
    }
    Ok(games)
}

pub fn reconstruct_game(game: &PgnGame) -> Result<GameRecord, PgnError> {
    let start_fen = game
        .tags
        .get("FEN")
        .cloned()
        .unwrap_or_else(|| OFFICIAL_STARTPOS_FEN.to_string());
    let mut position = parse_fen(&start_fen).map_err(|e| PgnError::InvalidInitialFen(format!("{e:?}")))?;
    let mut plies = Vec::new();

    for (idx, san) in game.moves.iter().enumerate() {
        let legal = generate_legal_moves(&position);
        let chosen = resolve_san(&position, &legal, san).ok_or_else(|| {
            PgnError::Replay(ReplayError {
                ply_index: idx + 1,
                san: san.clone(),
                message: "could not resolve SAN to a legal move".to_string(),
            })
        })?;
        apply_move(&mut position, chosen);
        plies.push(chosen);
    }

    Ok(GameRecord {
        game: game.clone(),
        position,
        plies,
    })
}

fn split_games(input: &str) -> Vec<String> {
    let mut games = Vec::new();
    let mut current = String::new();
    for line in input.lines() {
        if line.trim_start().starts_with("[Event ") && !current.trim().is_empty() {
            games.push(current.trim().to_string());
            current.clear();
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        games.push(current.trim().to_string());
    }
    games
}

fn parse_single_game(input: &str) -> Result<PgnGame, PgnError> {
    let mut tags = BTreeMap::new();
    let mut movetext = String::new();
    for line in input.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            let (k, v) = parse_tag_line(t)?;
            tags.insert(k, v);
        } else if !t.is_empty() {
            movetext.push_str(t);
            movetext.push(' ');
        }
    }

    let tokens = tokenize_movetext(&movetext);
    let mut moves = Vec::new();
    let mut result = None;
    for tok in tokens {
        if is_result_token(&tok) {
            result = Some(tok);
            continue;
        }
        if is_move_number_token(&tok) || tok == "*" {
            continue;
        }
        moves.push(tok);
    }

    Ok(PgnGame { tags, moves, result })
}

fn parse_tag_line(line: &str) -> Result<(String, String), PgnError> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return Err(PgnError::Parse(format!("invalid tag line: {line}")));
    }
    let inner = &line[1..line.len() - 1];
    let mut parts = inner.splitn(2, ' ');
    let key = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| PgnError::Parse(format!("missing tag key: {line}")))?;
    let raw_value = parts
        .next()
        .ok_or_else(|| PgnError::Parse(format!("missing tag value: {line}")))?;
    let value = raw_value.trim();
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        return Err(PgnError::Parse(format!("invalid tag value: {line}")));
    }
    Ok((key.to_string(), value[1..value.len() - 1].to_string()))
}

fn tokenize_movetext(input: &str) -> Vec<String> {
    let mut cleaned = String::new();
    let mut in_brace_comment = false;
    let mut paren_depth = 0usize;
    let mut in_line_comment = false;
    for ch in input.chars() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_brace_comment {
            if ch == '}' {
                in_brace_comment = false;
            }
            continue;
        }
        if paren_depth > 0 {
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
            }
            continue;
        }
        match ch {
            '{' => in_brace_comment = true,
            ';' => in_line_comment = true,
            '(' => paren_depth = 1,
            '$' => continue,
            '\n' | '\r' | '\t' => cleaned.push(' '),
            _ => cleaned.push(ch),
        }
    }
    cleaned
        .split_whitespace()
        .map(|s| s.trim().to_string())
        .collect()
}

fn is_result_token(tok: &str) -> bool {
    matches!(tok, "1-0" | "0-1" | "1/2-1/2" | "*")
}

fn is_move_number_token(tok: &str) -> bool {
    tok.ends_with('.') || tok.contains("...")
}

fn resolve_san(position: &Position, legal: &[ChessMove], san: &str) -> Option<ChessMove> {
    let clean = normalize_san(san);
    if clean == "O-O" {
        return legal
            .iter()
            .copied()
            .find(|m| m.castle == Some(CastleSide::KingSide));
    }
    if clean == "O-O-O" {
        return legal
            .iter()
            .copied()
            .find(|m| m.castle == Some(CastleSide::QueenSide));
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
        .filter(|mv| {
            position
                .piece_at(mv.from)
                .map(|p| p.kind == piece_kind)
                .unwrap_or(false)
        })
        .collect();

    if let Some(file) = from_file_hint {
        candidates.retain(|mv| mv.from.file() == file);
    }
    if let Some(rank) = from_rank_hint {
        candidates.retain(|mv| mv.from.rank() == rank);
    }
    candidates.into_iter().next()
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

pub fn color_from_result(result: Option<&String>) -> Option<Color> {
    match result.map(|s| s.as_str()) {
        Some("1-0") => Some(Color::White),
        Some("0-1") => Some(Color::Black),
        _ => None,
    }
}

mod parse;
mod replay;
mod san;

use std::collections::BTreeMap;
use std::io::BufRead;

use crate::board::{ChessMove, Color, Position};
use thiserror::Error;

pub use parse::{parse_pgn, parse_pgn_reader, PgnReader};
pub use replay::reconstruct_game;
pub use san::{move_to_san, resolve_san};

pub const OFFICIAL_STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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

#[derive(Debug, Clone, Error)]
#[error("ply {ply_index} `{san}`: {message}")]
pub struct ReplayError {
    pub ply_index: usize,
    pub san: String,
    pub message: String,
}

#[derive(Debug, Clone, Error)]
pub enum PgnError {
    #[error("PGN parse error: {0}")]
    Parse(String),
    #[error("invalid initial FEN: {0}")]
    InvalidInitialFen(String),
    #[error(transparent)]
    Replay(#[from] ReplayError),
}

pub fn color_from_result(result: Option<&String>) -> Option<Color> {
    match result.map(|s| s.as_str()) {
        Some("1-0") => Some(Color::White),
        Some("0-1") => Some(Color::Black),
        _ => None,
    }
}

pub(crate) fn read_to_games<R: BufRead>(reader: R) -> Result<Vec<PgnGame>, PgnError> {
    PgnReader::new(reader).collect()
}

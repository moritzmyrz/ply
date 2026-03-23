use crate::board::{ChessMove, PieceKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningInfo {
    pub eco: &'static str,
    pub opening: &'static str,
    pub variation: Option<&'static str>,
}

struct OpeningLine {
    info: OpeningInfo,
    line: &'static [&'static str],
}

const OPENINGS: &[OpeningLine] = &[
    OpeningLine {
        info: OpeningInfo { eco: "C60", opening: "Ruy Lopez", variation: None },
        line: &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "C50", opening: "Italian Game", variation: None },
        line: &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "B20", opening: "Sicilian Defense", variation: None },
        line: &["e2e4", "c7c5"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "C00", opening: "French Defense", variation: None },
        line: &["e2e4", "e7e6"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "B10", opening: "Caro-Kann Defense", variation: None },
        line: &["e2e4", "c7c6"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "D06", opening: "Queen's Gambit", variation: None },
        line: &["d2d4", "d7d5", "c2c4"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "D20", opening: "Queen's Gambit Accepted", variation: None },
        line: &["d2d4", "d7d5", "c2c4", "d5c4"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "D30", opening: "Queen's Gambit Declined", variation: None },
        line: &["d2d4", "d7d5", "c2c4", "e7e6"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "D10", opening: "Slav Defense", variation: None },
        line: &["d2d4", "d7d5", "c2c4", "c7c6"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "A00", opening: "Van't Kruijs Opening", variation: None },
        line: &["e2e3"],
    },
];

pub fn classify_opening(plies: &[ChessMove]) -> Option<OpeningInfo> {
    OPENINGS
        .iter()
        .filter(|opening| has_uci_prefix(plies, opening.line))
        .max_by_key(|opening| opening.line.len())
        .map(|opening| opening.info.clone())
}

/// Compare against UCI-like strings (`e2e4`, optional promotion suffix).
fn move_matches_uci(mv: ChessMove, uci: &str) -> bool {
    let mut buf = [0u8; 6];
    let n = write_move_uci(mv, &mut buf);
    uci.len() == n && uci.as_bytes() == &buf[..n]
}

fn write_move_uci(mv: ChessMove, out: &mut [u8; 6]) -> usize {
    out[0] = b'a' + mv.from.file();
    out[1] = b'1' + mv.from.rank();
    out[2] = b'a' + mv.to.file();
    out[3] = b'1' + mv.to.rank();
    if let Some(p) = mv.promotion {
        out[4] = match p {
            PieceKind::Knight => b'n',
            PieceKind::Bishop => b'b',
            PieceKind::Rook => b'r',
            PieceKind::Queen => b'q',
            PieceKind::Pawn | PieceKind::King => b'q',
        };
        return 5;
    }
    4
}

fn has_uci_prefix(plies: &[ChessMove], prefix: &[&str]) -> bool {
    plies.len() >= prefix.len()
        && plies.iter().zip(prefix.iter()).all(|(mv, expected)| move_matches_uci(*mv, expected))
}

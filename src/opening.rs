use crate::board::ChessMove;

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
        info: OpeningInfo {
            eco: "C50",
            opening: "Italian Game",
            variation: None,
        },
        line: &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "B20",
            opening: "Sicilian Defense",
            variation: None,
        },
        line: &["e2e4", "c7c5"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "C00",
            opening: "French Defense",
            variation: None,
        },
        line: &["e2e4", "e7e6"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "B10",
            opening: "Caro-Kann Defense",
            variation: None,
        },
        line: &["e2e4", "c7c6"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "D06",
            opening: "Queen's Gambit",
            variation: None,
        },
        line: &["d2d4", "d7d5", "c2c4"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "D20",
            opening: "Queen's Gambit Accepted",
            variation: None,
        },
        line: &["d2d4", "d7d5", "c2c4", "d5c4"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "D30",
            opening: "Queen's Gambit Declined",
            variation: None,
        },
        line: &["d2d4", "d7d5", "c2c4", "e7e6"],
    },
    OpeningLine {
        info: OpeningInfo { eco: "D10", opening: "Slav Defense", variation: None },
        line: &["d2d4", "d7d5", "c2c4", "c7c6"],
    },
    OpeningLine {
        info: OpeningInfo {
            eco: "A00",
            opening: "Van't Kruijs Opening",
            variation: None,
        },
        line: &["e2e3"],
    },
];

pub fn classify_opening(plies: &[ChessMove]) -> Option<OpeningInfo> {
    let coordinates = plies.iter().map(|mv| mv.to_coordinate()).collect::<Vec<_>>();
    OPENINGS
        .iter()
        .filter(|opening| has_prefix(&coordinates, opening.line))
        .max_by_key(|opening| opening.line.len())
        .map(|opening| opening.info.clone())
}

fn has_prefix(moves: &[String], prefix: &[&str]) -> bool {
    moves.len() >= prefix.len() && moves.iter().zip(prefix.iter()).all(|(mv, expected)| mv == expected)
}

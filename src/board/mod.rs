use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn from_fen(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'p' => Some(Self::Pawn),
            'n' => Some(Self::Knight),
            'b' => Some(Self::Bishop),
            'r' => Some(Self::Rook),
            'q' => Some(Self::Queen),
            'k' => Some(Self::King),
            _ => None,
        }
    }

    pub fn fen_char(self, color: Color) -> char {
        let c = match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        };
        match color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    pub fn san_char(self) -> Option<char> {
        match self {
            Self::Pawn => None,
            Self::Knight => Some('N'),
            Self::Bishop => Some('B'),
            Self::Rook => Some('R'),
            Self::Queen => Some('Q'),
            Self::King => Some('K'),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub u8);

impl Square {
    pub fn from_coords(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self(rank * 8 + file))
        } else {
            None
        }
    }

    pub fn file(self) -> u8 {
        self.0 % 8
    }

    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    pub fn from_algebraic(value: &str) -> Option<Self> {
        if value.len() != 2 {
            return None;
        }
        let bytes = value.as_bytes();
        let file = match bytes[0] {
            b'a'..=b'h' => bytes[0] - b'a',
            _ => return None,
        };
        let rank = match bytes[1] {
            b'1'..=b'8' => bytes[1] - b'1',
            _ => return None,
        };
        Self::from_coords(file, rank)
    }

    pub fn to_algebraic(self) -> String {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        format!("{file}{rank}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

impl CastlingRights {
    pub fn empty() -> Self {
        Self {
            white_king_side: false,
            white_queen_side: false,
            black_king_side: false,
            black_queen_side: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleSide {
    KingSide,
    QueenSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChessMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
    pub is_capture: bool,
    pub is_en_passant: bool,
    pub castle: Option<CastleSide>,
}

impl ChessMove {
    pub fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_capture: false,
            is_en_passant: false,
            castle: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Position {
    pub board: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant_target: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("side_to_move", &self.side_to_move)
            .field("castling", &self.castling)
            .field("en_passant_target", &self.en_passant_target)
            .field("halfmove_clock", &self.halfmove_clock)
            .field("fullmove_number", &self.fullmove_number)
            .finish()
    }
}

impl Position {
    pub fn empty() -> Self {
        Self {
            board: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::empty(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.board[sq.0 as usize]
    }

    pub fn set_piece(&mut self, sq: Square, piece: Option<Piece>) {
        self.board[sq.0 as usize] = piece;
    }

    pub fn king_square(&self, color: Color) -> Option<Square> {
        for idx in 0..64 {
            if let Some(piece) = self.board[idx] {
                if piece.color == color && piece.kind == PieceKind::King {
                    return Some(Square(idx as u8));
                }
            }
        }
        None
    }
}

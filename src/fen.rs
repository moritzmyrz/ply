use crate::board::{CastlingRights, Color, Piece, PieceKind, Position, Square};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    InvalidFieldCount,
    InvalidBoard(String),
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidHalfmoveClock,
    InvalidFullmoveNumber,
}

pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn parse_fen(input: &str) -> Result<Position, FenError> {
    let fields: Vec<&str> = input.split_whitespace().collect();
    if fields.len() != 6 {
        return Err(FenError::InvalidFieldCount);
    }

    let mut position = Position::empty();
    parse_board(fields[0], &mut position)?;
    position.side_to_move = parse_side_to_move(fields[1])?;
    position.castling = parse_castling(fields[2])?;
    position.en_passant_target = parse_en_passant(fields[3])?;
    position.halfmove_clock =
        fields[4].parse::<u32>().map_err(|_| FenError::InvalidHalfmoveClock)?;
    position.fullmove_number =
        fields[5].parse::<u32>().map_err(|_| FenError::InvalidFullmoveNumber)?;
    if position.fullmove_number == 0 {
        return Err(FenError::InvalidFullmoveNumber);
    }
    Ok(position)
}

pub fn to_fen(position: &Position) -> String {
    let mut board_str = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            let sq = Square::from_coords(file, rank).expect("square");
            if let Some(piece) = position.piece_at(sq) {
                if empty > 0 {
                    board_str.push_str(&empty.to_string());
                    empty = 0;
                }
                board_str.push(piece.kind.fen_char(piece.color));
            } else {
                empty += 1;
            }
        }
        if empty > 0 {
            board_str.push_str(&empty.to_string());
        }
        if rank > 0 {
            board_str.push('/');
        }
    }

    let side = match position.side_to_move {
        Color::White => "w",
        Color::Black => "b",
    };

    let mut castling = String::new();
    if position.castling.white_king_side {
        castling.push('K');
    }
    if position.castling.white_queen_side {
        castling.push('Q');
    }
    if position.castling.black_king_side {
        castling.push('k');
    }
    if position.castling.black_queen_side {
        castling.push('q');
    }
    if castling.is_empty() {
        castling.push('-');
    }

    let en_passant =
        position.en_passant_target.map(|sq| sq.to_algebraic()).unwrap_or_else(|| "-".to_string());

    format!(
        "{board_str} {side} {castling} {en_passant} {} {}",
        position.halfmove_clock, position.fullmove_number
    )
}

fn parse_board(field: &str, position: &mut Position) -> Result<(), FenError> {
    let ranks: Vec<&str> = field.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::InvalidBoard("expected 8 ranks".to_string()));
    }
    for (fen_rank, chunk) in ranks.iter().enumerate() {
        let rank = 7 - fen_rank as u8;
        let mut file = 0u8;
        for ch in chunk.chars() {
            if ch.is_ascii_digit() {
                let n = ch
                    .to_digit(10)
                    .ok_or_else(|| FenError::InvalidBoard("invalid digit".to_string()))?
                    as u8;
                if n == 0 || n > 8 {
                    return Err(FenError::InvalidBoard("digit out of range".to_string()));
                }
                file += n;
                continue;
            }
            let kind = PieceKind::from_fen(ch)
                .ok_or_else(|| FenError::InvalidBoard(format!("invalid piece: {ch}")))?;
            if file >= 8 {
                return Err(FenError::InvalidBoard("file overflow".to_string()));
            }
            let color = if ch.is_ascii_uppercase() { Color::White } else { Color::Black };
            let sq = Square::from_coords(file, rank).expect("square");
            position.set_piece(sq, Some(Piece { color, kind }));
            file += 1;
        }
        if file != 8 {
            return Err(FenError::InvalidBoard("rank not complete".to_string()));
        }
    }
    Ok(())
}

fn parse_side_to_move(field: &str) -> Result<Color, FenError> {
    match field {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenError::InvalidSideToMove),
    }
}

fn parse_castling(field: &str) -> Result<CastlingRights, FenError> {
    if field == "-" {
        return Ok(CastlingRights::empty());
    }
    let mut rights = CastlingRights::empty();
    for ch in field.chars() {
        match ch {
            'K' => rights.white_king_side = true,
            'Q' => rights.white_queen_side = true,
            'k' => rights.black_king_side = true,
            'q' => rights.black_queen_side = true,
            _ => return Err(FenError::InvalidCastlingRights),
        }
    }
    Ok(rights)
}

fn parse_en_passant(field: &str) -> Result<Option<Square>, FenError> {
    if field == "-" {
        return Ok(None);
    }
    let sq = Square::from_algebraic(field).ok_or(FenError::InvalidEnPassant)?;
    let rank = sq.rank();
    if rank != 2 && rank != 5 {
        return Err(FenError::InvalidEnPassant);
    }
    Ok(Some(sq))
}

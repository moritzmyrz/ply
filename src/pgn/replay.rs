use crate::fen::parse_fen;
use crate::movegen::{apply_move, generate_legal_moves};

use super::san::resolve_san;
use super::{GameRecord, OFFICIAL_STARTPOS_FEN, PgnError, PgnGame, ReplayError};

pub fn reconstruct_game(game: &PgnGame) -> Result<GameRecord, PgnError> {
    let start_fen =
        game.tags.get("FEN").cloned().unwrap_or_else(|| OFFICIAL_STARTPOS_FEN.to_string());
    let mut position = parse_fen(&start_fen).map_err(|e| PgnError::InvalidInitialFen(e.to_string()))?;
    let mut plies = Vec::new();

    for (idx, san) in game.moves.iter().enumerate() {
        let legal = generate_legal_moves(&position);
        let chosen = resolve_san(&position, &legal, san).ok_or_else(|| ReplayError {
            ply_index: idx + 1,
            san: san.clone(),
            message: "could not resolve SAN to a legal move".to_string(),
        })?;
        apply_move(&mut position, chosen);
        plies.push(chosen);
    }

    Ok(GameRecord { game: game.clone(), position, plies })
}

use crate::board::Board;
use crate::chess::piece::Piece;
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::Pawn;

/// Amount of cp a single pawn advancement should be worth
const ROW_WORTH: i32 = 10;

/// Pushing pawns is generally a good idea
/// Counts the score of advancement of whites pawns vs black pawns
pub fn push_pawns(board: &Board) -> i32 {
    let mut eval = 0;
    let white_pawns = board.get_bitboard(Piece::new(Pawn, White));
    let black_pawns = board.get_bitboard(Piece::new(Pawn, Black));

    for sq in white_pawns.iter() {
        eval += sq.row() as i32 * ROW_WORTH;
    }
    for sq in black_pawns.iter() {
        eval -= sq.row() as i32 * ROW_WORTH;
    }

    eval
}

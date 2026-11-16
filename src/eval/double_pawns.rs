use crate::board::Board;
use crate::chess::piece::Piece;
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::Pawn;

const COL_A: u64 = 0x0101010101010101;
/// Penalty for each doubled pawn
const PENALTY: i32 = -50;

/// Having double pawns is usually bad
/// This heuristic assigns a penalty to each double pawn to reflect this
pub fn double_pawns(board: &Board) -> i32 {
    let mut count_white = 0;
    let pawns_white = board.get_bitboard(Piece::new(Pawn, White));
    for col in 0..8 {
        let n = (pawns_white.to_u64() & (COL_A << col)).count_ones() as i32;
        count_white += (n - 1).max(0);
    }

    let mut count_black = 0;
    let pawns_black = board.get_bitboard(Piece::new(Pawn, Black));
    for col in 0..8 {
        let n = (pawns_black.to_u64() & (COL_A << col)).count_ones() as i32;
        count_black += (n - 1).max(0);
    }

    (count_white - count_black) * PENALTY
}

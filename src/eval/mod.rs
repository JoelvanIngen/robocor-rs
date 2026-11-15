mod piece_values;
mod push_pawns;

use crate::board::Board;
use crate::eval::piece_values::piece_values;
use crate::eval::push_pawns::push_pawns;

/// Receives a board and computes an evaluation for white in centipawns
/// Blacks evaluation would be -eval
pub fn eval_pos(board: &Board) -> i32 {
    let mut eval = 0;
    eval += piece_values(board);
    eval += push_pawns(board);
    eval
}

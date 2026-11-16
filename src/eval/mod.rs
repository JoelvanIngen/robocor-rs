mod double_pawns;
mod mobility;
mod piece_values;
mod push_pawns;
mod square_attacks;

use crate::board::Board;
use crate::eval::double_pawns::double_pawns;
use crate::eval::mobility::mobility;
use crate::eval::piece_values::piece_values;
use crate::eval::push_pawns::push_pawns;
use crate::eval::square_attacks::square_attacks;

/// Receives a board and computes an evaluation for white in centipawns
/// Black's evaluation would be -eval
pub fn eval_pos(board: &Board) -> i32 {
    piece_values(board)
        + push_pawns(board)
        + mobility(board)
        + double_pawns(board)
        + square_attacks(board)
}

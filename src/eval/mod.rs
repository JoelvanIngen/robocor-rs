mod piece_values;

use crate::board::Board;
use crate::eval::piece_values::piece_values;

/// Receives a board and computes an evaluation for white
/// Blacks evaluation would be -0
pub fn eval_pos(board: &Board) -> f32 {
    let mut eval = 0f32;
    eval += piece_values(board);
    eval
}

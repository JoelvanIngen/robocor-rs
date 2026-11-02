use crate::board::moves::MoveKind::{Capture, QuietMove};
use crate::board::moves::{Move, MoveList};
use crate::board::{BitBoard, Square};

/// Iterate in a given direction until stopped or reaching the end of the board
/// Returns all pseudo legal moves in that direction
pub fn get_pl_moves_direction(
    moves: &mut MoveList,
    orig_pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    dir: (i8, i8),
) {
    let mut pos = orig_pos;
    loop {
        let new_row = pos.row() as i8 + dir.0;
        let new_col = pos.col() as i8 + dir.1;
        pos = match is_within_board(new_row, new_col) {
            true => Square::from_coords(new_row as usize, new_col as usize),
            false => return,
        };

        if friendly.is_set(pos) {
            return;
        }

        if enemy.is_set(pos) {
            moves.push(Move::new(Capture, orig_pos, pos));
            return;
        }

        moves.push(Move::new(QuietMove, orig_pos, pos));
    }
}

fn is_within_board(row: i8, col: i8) -> bool {
    (0..8).contains(&row) && (0..8).contains(&col)
}

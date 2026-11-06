use crate::board::{BitBoard, Square};
use crate::chess::moves::MoveKind::{Capture, QuietMove};
use crate::chess::moves::{Move, MoveList};

pub fn get_attacks_sliding(
    pos: Square,
    occupied: BitBoard,
    directions: &[(i8, i8); 4],
) -> BitBoard {
    let mut attacks = BitBoard::EMPTY;
    let start_row = pos.row() as i8;
    let start_col = pos.col() as i8;

    for (dr, dc) in directions {
        let mut r = start_row + dr;
        let mut c = start_col + dc;

        while (0..8).contains(&r) && (0..8).contains(&c) {
            let sq_idx = (r * 8 + c) as usize;
            attacks.set_idx(sq_idx);

            if occupied.is_set_idx(sq_idx) {
                break;
            }
            r += dr;
            c += dc;
        }
    }
    attacks
}

pub fn get_pl_moves_sliding(
    moves: &mut MoveList,
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    directions: &[(i8, i8); 4],
) {
    let start_row = pos.row() as i8;
    let start_col = pos.col() as i8;

    for (dr, dc) in directions {
        let mut r = start_row + dr;
        let mut c = start_col + dc;

        while (0..8).contains(&r) && (0..8).contains(&c) {
            let sq_idx = (r * 8 + c) as usize;
            if friendly.is_set_idx(sq_idx) {
                break;
            }
            if enemy.is_set_idx(sq_idx) {
                moves.push(Move::new(Capture, pos, Square::from_index(sq_idx)));
                break;
            }

            moves.push(Move::new(QuietMove, pos, Square::from_index(sq_idx)));
            r += dr;
            c += dc;
        }
    }
}

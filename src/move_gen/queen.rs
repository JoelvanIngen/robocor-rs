use crate::board::{BitBoard, Square};
use crate::chess::moves::MoveList;
use crate::move_gen::bishop::{get_attacks_bishop, get_pl_moves_bishop};
use crate::move_gen::rook::{get_attacks_rook, get_pl_moves_rook};

pub fn get_attacks_queen(pos: Square, friendly: BitBoard, enemy: BitBoard) -> BitBoard {
    get_attacks_rook(pos, friendly, enemy) | get_attacks_bishop(pos, friendly, enemy)
}

pub fn get_pl_moves_queen(moves: &mut MoveList, pos: Square, friendly: BitBoard, enemy: BitBoard) {
    get_pl_moves_rook(moves, pos, friendly, enemy);
    get_pl_moves_bishop(moves, pos, friendly, enemy);
}

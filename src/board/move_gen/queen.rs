use crate::board::move_gen::bishop::{get_attacks_bishop, get_pl_moves_bishop};
use crate::board::move_gen::rook::{get_attacks_rook, get_pl_moves_rook};
use crate::board::moves::Move;
use crate::board::{BitBoard, Square};

pub fn get_attacks_queen(pos: Square, friendly: BitBoard, enemy: BitBoard) -> BitBoard {
    get_attacks_rook(pos, friendly, enemy) & get_attacks_bishop(pos, friendly, enemy)
}

pub fn get_pl_moves_queen(pos: Square, friendly: BitBoard, enemy: BitBoard) -> Vec<Move> {
    let mut moves = get_pl_moves_rook(pos, friendly, enemy);
    moves.extend(get_pl_moves_bishop(pos, friendly, enemy));
    moves
}

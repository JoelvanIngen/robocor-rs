use crate::board::bitboard::BitBoard;
use crate::board::moves::Move;
use crate::board::piece::PieceKind;

const KING_DELTAS: [(i8, i8); 8] = [
    (-1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

impl PieceKind {
    pub fn generate_pl_moves(&self, friendly: BitBoard, enemy: BitBoard) -> Vec<Move> {
        let moves = vec![];

        moves
    }
}

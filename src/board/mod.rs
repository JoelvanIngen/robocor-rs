mod attacks;
mod display;
mod fen;
mod hashing;
mod move_generation;
mod position;

pub use crate::chess::bitboard::BitBoard;
use crate::chess::piece::{PieceColour, PieceKind};
pub use crate::chess::square::Square;
pub use crate::move_gen::get_pl_moves_knight;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    // 2D array bitboard[colour][piece]
    bitboards: [[BitBoard; PieceKind::COUNT]; PieceColour::COUNT],
    turn: PieceColour,
    // Bit 1-4 unused, 5 WQ, 6 WK, 7 BQ, 8 BK
    castling_rights: u8,
    en_passant: BitBoard,
    halfmove_clock: u8, // Max value 50
    fullmove_number: u16,

    colour_bitboards: [BitBoard; 2],

    hash: u64,
}

fn combine_bitboards(bitboards: [BitBoard; 6]) -> BitBoard {
    let mut res = BitBoard::EMPTY;
    for bb in bitboards {
        res |= bb;
    }
    res
}

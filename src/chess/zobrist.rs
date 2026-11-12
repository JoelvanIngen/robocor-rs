use crate::board::{BitBoard, Square};
use crate::chess::piece::PieceColour::Black;
use crate::chess::piece::{PieceColour, PieceKind};

pub struct ZobristKeys {
    pub pieces: [[[u64; 64]; PieceKind::COUNT]; PieceColour::COUNT],
    pub side_to_move: u64,
    pub castling_rights: [u64; 16], // 16 possible combinations
    pub en_passant: [u64; 8],
}

/// https://rosettacode.org/wiki/Pseudo-random_numbers/Splitmix64
const fn splitmix64(mut state: u64) -> (u64, u64) {
    state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    (state, z ^ (z >> 31))
}

pub const ZOBRIST: ZobristKeys = {
    // Determined by keysmash
    let mut state = 0x1267FB5387C5BB35;

    let mut pieces = [[[0u64; 64]; PieceKind::COUNT]; PieceColour::COUNT];
    let mut colour_idx = 0;
    while colour_idx < PieceColour::COUNT {
        let mut kind_idx = 0;
        while kind_idx < PieceKind::COUNT {
            let mut square_idx = 0;
            while square_idx < 64 {
                let (new_state, rand) = splitmix64(state);
                state = new_state;
                pieces[colour_idx][kind_idx][square_idx] = rand;
                square_idx += 1;
            }
            kind_idx += 1;
        }
        colour_idx += 1;
    }

    let (new_state, side_to_move) = splitmix64(state);
    state = new_state;

    let mut castling_rights = [0; 16];
    let mut i = 0;
    while i < 16 {
        let (new_state, rand) = splitmix64(state);
        state = new_state;
        castling_rights[i] = rand;
        i += 1;
    }

    let mut en_passant = [0; 8];
    let mut i = 0;
    while i < 8 {
        let (_, rand) = splitmix64(state);
        en_passant[i] = rand;
        i += 1;
    }

    ZobristKeys {
        pieces,
        side_to_move,
        castling_rights,
        en_passant,
    }
};

pub fn calculate_hash(
    bitboards: [[BitBoard; PieceKind::COUNT]; PieceColour::COUNT],
    turn: PieceColour,
    castling_rights: u8,
    en_passant: BitBoard,
) -> u64 {
    let mut hash = 0u64;

    for colour in PieceColour::ALL {
        for kind in PieceKind::ALL {
            for sq in bitboards[colour as usize][kind as usize].iter() {
                hash ^= ZOBRIST.pieces[colour as usize][kind as usize][sq.idx()];
            }
        }
    }

    if turn == Black {
        hash ^= ZOBRIST.side_to_move;
    }

    hash ^= ZOBRIST.castling_rights[castling_rights as usize];

    if en_passant != BitBoard::EMPTY {
        let sq = en_passant.get_square();
        hash ^= ZOBRIST.en_passant[sq.col()];
    }

    hash
}

/// Binds castling squares combination to (0..16) index
pub fn castling_index(bb: BitBoard) -> usize {
    (bb.is_set(Square::from_coords(0, 0)) as usize)
        | ((bb.is_set(Square::from_coords(0, 7)) as usize) << 1)
        | ((bb.is_set(Square::from_coords(7, 0)) as usize) << 2)
        | ((bb.is_set(Square::from_coords(7, 7)) as usize) << 3)
}

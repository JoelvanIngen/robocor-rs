//! Some squares are worth more to attack than others
//! We like grabbing control of the midfield, but don't care as much
//! about corners or edges
//! This is a static implementation, but in reality these values would vary
//! over the opening, midgame, and endgame

use crate::board::{BitBoard, Board};
use crate::chess::piece::Piece;
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::{Bishop, Knight, Pawn, Queen, Rook};
use crate::move_gen::{
    get_attacks_bishop, get_attacks_knight, get_attacks_pawn, get_attacks_queen, get_attacks_rook,
};

/// For interpretation, these grids should be flipped vertically
/// Internal repr counts from bottom left to top right,
/// but code starts top left and ends bottom right
/// These values have been selected based on guessing
const SQUARE_ATTACK_VALUE_WHITE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // Comment to prevent autoformatting
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 1, 2, 2, 1, 0, 0, //
    0, 1, 1, 3, 3, 1, 1, 0, //
    0, 1, 2, 4, 4, 2, 1, 0, //
    1, 1, 2, 4, 4, 2, 1, 1, //
    1, 1, 2, 5, 5, 2, 1, 1, //
    4, 5, 6, 6, 6, 6, 5, 4, //
];
const SQUARE_ATTACK_VALUE_BLACK: [i32; 64] = {
    let mut array = SQUARE_ATTACK_VALUE_WHITE;
    array.reverse();
    array
};

/// Piece multipliers
/// Ideally we would implement different tables for each piece
/// but that is a lot of work for now
/// We care more about a far-advanced pawn (or taking our opponent's)
/// than a queen that happens to attack some far-away square
const PAWN_MULT: i32 = 10;
const KNIGHT_MULT: i32 = 5;
const BISHOP_MULT: i32 = 5;
const ROOK_MULT: i32 = 5;
const QUEEN_MULT: i32 = 2;

fn weighted_sum(mut mask: u64, values: &[i32; 64]) -> i32 {
    let mut sum = 0;
    while mask != 0 {
        let idx = mask.trailing_zeros() as usize;
        sum += values[idx];
        mask &= mask - 1; // clear lowest set bit
    }
    sum
}

fn pawn_attacks(board: &Board) -> i32 {
    let mut score = 0;

    let white_pawns = board.get_bitboard(Piece::new(Pawn, White));
    for pos in white_pawns.iter() {
        let mask = get_attacks_pawn(pos, White);
        score += weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_WHITE);
    }

    let black_pawns = board.get_bitboard(Piece::new(Pawn, Black));
    for pos in black_pawns.iter() {
        let mask = get_attacks_pawn(pos, Black);
        score -= weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_BLACK);
    }

    score * PAWN_MULT
}

fn knight_attacks(board: &Board) -> i32 {
    let mut score = 0;

    let white_knights = board.get_bitboard(Piece::new(Knight, White));
    for pos in white_knights.iter() {
        let mask = get_attacks_knight(pos);
        score += weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_WHITE);
    }

    let white_knights = board.get_bitboard(Piece::new(Knight, Black));
    for pos in white_knights.iter() {
        let mask = get_attacks_knight(pos);
        score -= weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_BLACK);
    }

    score * KNIGHT_MULT
}

fn bishop_attacks(board: &Board, white: BitBoard, black: BitBoard) -> i32 {
    let mut score = 0;

    let white_bishops = board.get_bitboard(Piece::new(Bishop, White));
    for pos in white_bishops.iter() {
        let mask = get_attacks_bishop(pos, white, black);
        score += weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_WHITE);
    }

    let black_bishops = board.get_bitboard(Piece::new(Bishop, Black));
    for pos in black_bishops.iter() {
        let mask = get_attacks_bishop(pos, black, white);
        score -= weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_BLACK);
    }

    score * BISHOP_MULT
}

fn rook_attacks(board: &Board, white: BitBoard, black: BitBoard) -> i32 {
    let mut score = 0;

    let white_rooks = board.get_bitboard(Piece::new(Rook, White));
    for pos in white_rooks.iter() {
        let mask = get_attacks_rook(pos, white, black);
        score += weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_WHITE);
    }

    let black_rooks = board.get_bitboard(Piece::new(Rook, Black));
    for pos in black_rooks.iter() {
        let mask = get_attacks_rook(pos, black, white);
        score -= weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_BLACK);
    }

    score * ROOK_MULT
}

fn queen_attacks(board: &Board, white: BitBoard, black: BitBoard) -> i32 {
    let mut score = 0;

    let white_queens = board.get_bitboard(Piece::new(Queen, White));
    for pos in white_queens.iter() {
        let mask = get_attacks_queen(pos, white, black);
        score += weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_WHITE);
    }

    let black_queens = board.get_bitboard(Piece::new(Queen, Black));
    for pos in black_queens.iter() {
        let mask = get_attacks_queen(pos, black, white);
        score -= weighted_sum(mask.to_u64(), &SQUARE_ATTACK_VALUE_BLACK);
    }

    score * QUEEN_MULT
}

pub fn square_attacks(board: &Board) -> i32 {
    let white_occ = board.get_colour_bitboard(White);
    let black_occ = board.get_colour_bitboard(Black);

    pawn_attacks(board)
        + knight_attacks(board)
        + bishop_attacks(board, white_occ, black_occ)
        + rook_attacks(board, white_occ, black_occ)
        + queen_attacks(board, white_occ, black_occ)
}

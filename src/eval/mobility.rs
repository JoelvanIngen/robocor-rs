use crate::board::Board;
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::{Bishop, Knight, Queen, Rook};
use crate::chess::piece::{Piece, PieceColour};
use crate::move_gen::{
    get_attacks_bishop, get_attacks_knight, get_attacks_queen, get_attacks_rook,
};

/// Score per square reachable for each piece type
/// These values have been chosen based on vibes
/// Knights are useless in corners, so high score
/// Bishops get stuck easily but can also reach many squares, so moderate high score
/// Rooks and especially queens are good at maneuvering out of tricky positions,
/// so lower score
const KNIGHT_MOBILITY: i32 = 50;
const BISHOP_MOBILITY: i32 = 20;
const ROOK_MOBILITY: i32 = 15;
const QUEEN_MOBILITY: i32 = 10;

/// Creating positions where you have a lot of mobility and the enemy's
/// pieces are trapped, is usually good. This also encourages opening up
/// pieces and lines in the opening
/// Adjusted for colour
pub fn mobility(board: &Board) -> i32 {
    let mut eval = 0;

    eval += mobility_knight(board, White);
    eval += mobility_bishop(board, White);
    eval += mobility_rook(board, White);
    eval += mobility_queen(board, White);

    eval -= mobility_knight(board, Black);
    eval -= mobility_bishop(board, Black);
    eval -= mobility_rook(board, Black);
    eval -= mobility_queen(board, Black);

    eval
}

/// Not adjusted for colour
fn mobility_knight(board: &Board, colour: PieceColour) -> i32 {
    let mut eval = 0;
    for pos in board.get_bitboard(Piece::new(Knight, colour)).iter() {
        eval += get_attacks_knight(pos).count() as i32 * KNIGHT_MOBILITY
    }
    eval
}

/// Not adjusted for colour
fn mobility_bishop(board: &Board, colour: PieceColour) -> i32 {
    let mut eval = 0;
    let friendly = board.get_colour_bitboard(colour);
    let enemy = board.get_colour_bitboard(colour.other());
    for pos in board.get_bitboard(Piece::new(Bishop, colour)).iter() {
        eval += get_attacks_bishop(pos, friendly, enemy).count() as i32 * BISHOP_MOBILITY
    }
    eval
}

/// Not adjusted for colour
fn mobility_rook(board: &Board, colour: PieceColour) -> i32 {
    let mut eval = 0;
    let friendly = board.get_colour_bitboard(colour);
    let enemy = board.get_colour_bitboard(colour.other());
    for pos in board.get_bitboard(Piece::new(Rook, colour)).iter() {
        eval += get_attacks_rook(pos, friendly, enemy).count() as i32 * ROOK_MOBILITY
    }
    eval
}

/// Not adjusted for colour
fn mobility_queen(board: &Board, colour: PieceColour) -> i32 {
    let mut eval = 0;
    let friendly = board.get_colour_bitboard(colour);
    let enemy = board.get_colour_bitboard(colour.other());
    for pos in board.get_bitboard(Piece::new(Queen, colour)).iter() {
        eval += get_attacks_queen(pos, friendly, enemy).count() as i32 * QUEEN_MOBILITY
    }
    eval
}

use crate::board::{BitBoard, Board, Square, get_pl_moves_knight};

mod board;
mod error;

fn main() {
    let board = Board::start_position();

    // println!("{:?}", board);

    // println!("{}", board);

    println!("{}", crate::board::KNIGHT_MOVES[4 * 8 + 4]);
    println!(
        "{:?}",
        get_pl_moves_knight(Square::from_coords(4, 4), BitBoard::EMPTY, BitBoard::EMPTY)
    );
}

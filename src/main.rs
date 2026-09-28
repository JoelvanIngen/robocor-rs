use crate::board::Board;

mod board;
mod error;

fn main() {
    let board = Board::start_position();

    println!("{:?}", board);

    println!("{}", board)
}

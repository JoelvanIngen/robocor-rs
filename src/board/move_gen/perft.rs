use crate::board::Board;

pub fn perft(depth: u8) {
    let board = Board::start_position();
    let n_moves = calc_positions(board, depth);
    println!("{}", n_moves);
}

/// Recursively calculates the number of positions
fn calc_positions(board: Board, remaining_depth: u8) -> usize {
    if remaining_depth == 0 {
        return 1;
    }

    let mut move_number = 0usize;

    for m in board.generate_moves() {
        let mut m_board = board;
        m_board.apply_move(m);
        move_number += calc_positions(m_board, remaining_depth - 1);
    }

    move_number
}

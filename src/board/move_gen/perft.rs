use crate::board::Board;
use std::time::Instant;

pub fn perft(depth: u8) {
    let board = Board::start_position();

    let start_time = Instant::now();
    let n_moves = calc_positions(board, depth);
    let end_time = Instant::now();

    let duration = end_time - start_time;

    println!(
        "{} in {}ms ({} nodes/second)",
        n_moves,
        duration.as_micros() as f32 / 1000f32,
        n_moves as f32 / duration.as_secs_f32()
    );
}

/// Recursively calculates the number of positions
fn calc_positions(board: Board, remaining_depth: u8) -> usize {
    if remaining_depth == 0 {
        return 1;
    }

    let moves = board.generate_moves();
    if remaining_depth == 1 {
        return moves.len();
    }

    let mut move_number = 0usize;

    for m in moves {
        let mut m_board = board;
        m_board.apply_move(m);

        move_number += calc_positions(m_board, remaining_depth - 1);
    }

    move_number
}

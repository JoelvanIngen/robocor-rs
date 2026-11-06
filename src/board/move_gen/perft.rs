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

/// Tests perft test results against known values
/// https://chessprogramming.org/Perft_Results
#[cfg(test)]
mod tests {
    use super::*;

    fn perform_perft_test(fen: &str, depth: u8, expected: usize) {
        let board = Board::from_fen(fen).expect(format!("Failed to parse fen {}", fen).as_str());
        let res = calc_positions(board, depth);
        assert_eq!(
            res, expected,
            "Expected to find {expected} positions, found {res}"
        );
    }

    #[test]
    fn pos1() {
        perform_perft_test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            5,
            4865609,
        )
    }

    #[test]
    fn pos2() {
        perform_perft_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            4,
            4085603,
        )
    }

    #[test]
    fn pos3() {
        perform_perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674624)
    }

    #[test]
    fn pos4() {
        perform_perft_test(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            4,
            422333,
        )
    }

    #[test]
    fn pos4_mirrored() {
        perform_perft_test(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
            4,
            422333,
        )
    }

    #[test]
    fn pos5() {
        perform_perft_test(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            4,
            2103487,
        )
    }

    #[test]
    fn pos6() {
        perform_perft_test(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            4,
            3894594,
        )
    }
}

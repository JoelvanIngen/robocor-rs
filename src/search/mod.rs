use crate::board::Board;
use crate::chess::moves::Move;
use crate::chess::piece::PieceColour::White;
use crate::eval::eval_pos;

const INF: i32 = 1_000_000;
const MATE: i32 = 100_000;

/// https://chessprogramming.org/Negamax
pub fn best_move(board: &Board, depth: u8) -> Option<Move> {
    let moves = board.generate_moves();
    if moves.is_empty() {
        return None;
    }

    let mut best = moves[0];
    let mut alpha = -INF;
    let beta = INF;

    for m in moves {
        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(&next_board, depth - 1, -beta, -alpha, 1);

        if score > alpha {
            alpha = score;
        }
    }

    Some(best)
}

/// https://chessprogramming.org/Negamax
fn negamax(board: &Board, depth: u8, mut alpha: i32, beta: i32, ply: u8) -> i32 {
    let moves = board.generate_moves();
    if moves.is_empty() {
        return if board.is_in_check() {
            // Delay getting mated
            -MATE + ply as i32
        } else {
            // Stalemate
            0
        };
    };

    if depth == 0 {
        let eval = eval_pos(board);
        return if board.turn() == White { eval } else { -eval };
    }

    for m in moves {
        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(&next_board, depth - 1, -beta, -alpha, ply + 1);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

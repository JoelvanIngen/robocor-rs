use crate::board::Board;
use crate::chess::moves::Move;
use crate::chess::moves::MoveKind::EnPassant;
use crate::chess::piece::PieceColour::White;
use crate::eval::eval_pos;
use crate::search::{MAX_PLY, SearchContext, pick_move, piece_value, score_moves};

/// Evaluates board from side to move
#[inline(always)]
fn side_eval(board: &Board) -> i32 {
    let eval = eval_pos(board);
    if board.turn() == White { eval } else { -eval }
}
/// Only search captures and promotions until only quiet moves are available
/// Prevents search cut-off in the middle of piece exchange
pub fn quiescence(
    board: &Board,
    ctx: &mut SearchContext,
    mut alpha: i32,
    beta: i32,
    ply: u8,
) -> i32 {
    ctx.nodes += 1;

    let do_nothing = side_eval(board);
    if ply as usize >= MAX_PLY - 1 {
        return do_nothing;
    }
    if do_nothing >= beta {
        return beta;
    }
    if do_nothing > alpha {
        alpha = do_nothing;
    }

    // TODO(Opt): Add `board.generate_captures()` to avoid generating moves that we
    //            will immediately discard
    let mut captures = board.generate_moves();
    captures.retain(|m| m.is_capture());

    let mut scores = score_moves(board, &captures, Move::NULL, ctx, ply as usize);

    for i in 0..captures.len() {
        pick_move(&mut captures, &mut scores, i);
        let m = captures[i];

        // Delta pruning
        // https://chessprogramming.org/Delta_Pruning
        if !m.is_promotion() {
            let captured_val = if m.kind() == EnPassant {
                100
            } else {
                piece_value(board.get_piece(m.to()).unwrap().kind)
            };
            if do_nothing + captured_val + 200 < alpha {
                continue;
            }
        }

        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -quiescence(&next_board, ctx, -beta, -alpha, ply + 1);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

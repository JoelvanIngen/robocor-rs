use crate::board::Board;
use crate::chess::moves::Move;
use crate::chess::moves::MoveKind::EnPassant;
use crate::chess::piece::PieceColour::White;
use crate::eval::eval_pos;
use crate::search::{MAX_PLY, SearchContext, SearchFrame, pick_move, piece_value, score_moves};

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
    search_frames: &mut [SearchFrame],
    mut alpha: i32,
    beta: i32,
    ply: u8,
) -> i32 {
    let (sf, sf_rest) = search_frames.split_first_mut().unwrap();
    let moves = &mut sf.moves;
    let scores = &mut sf.scores;

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
    board.generate_moves(moves);
    // `data.moves` will only contain captures after this point
    moves.retain(|m| m.is_capture());

    score_moves(scores, board, moves, Move::NULL, ctx, ply as usize);

    for i in 0..moves.len() {
        pick_move(moves, scores, i);
        let m = moves[i];

        // Delta pruning
        // https://chessprogramming.org/Delta_Pruning
        if !m.is_promotion() {
            let captured_val = if m.kind() == EnPassant {
                100
            } else {
                piece_value(board.get_colour_piece(m.to(), board.turn().other()).kind)
            };
            if do_nothing + captured_val + 200 < alpha {
                continue;
            }
        }

        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -quiescence(&next_board, ctx, sf_rest, -beta, -alpha, ply + 1);

        if ctx.aborted {
            return alpha;
        }

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

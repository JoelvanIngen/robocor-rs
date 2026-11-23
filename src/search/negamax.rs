use crate::board::Board;
use crate::chess::moves::Move;
use crate::search::lmr::lmr_reduction;
use crate::search::quiescence::quiescence;
use crate::search::transposition::{TTFlag, score_from_tt, score_to_tt};
use crate::search::{
    MATE, MATE_THRESHOLD, MAX_HISTORY, MAX_PLY, SearchContext, SearchFrame, pick_move, score_moves,
};

/// https://chessprogramming.org/Negamax
pub fn negamax(
    board: &Board,
    ctx: &mut SearchContext,
    search_frames: &mut [SearchFrame],
    depth: u8,
    mut alpha: i32,
    beta: i32,
    ply: u8,
    is_null_move: bool,
    is_pv: bool,
) -> i32 {
    let (sf, sf_rest) = search_frames.split_first_mut().unwrap();
    let moves = &mut sf.moves;
    let scores = &mut sf.scores;

    let ply_idx = ply as usize;
    ctx.pv_len[ply_idx] = 0; // Reset PV from prior branches

    // Draw checks
    if ctx.rep.is_repetition(board.halfmove_clock()) {
        return 0;
    }
    if board.is_fifty_move_draw() {
        if board.is_in_check() {
            // Not draw if checkmate
            board.generate_moves(moves);
            if moves.is_empty() {
                return -MATE + ply as i32;
            }
        }
        return 0;
    }
    if !board.sufficient_material() {
        return 0;
    }

    if depth == 0 || ply_idx >= MAX_PLY - 1 {
        return quiescence(board, ctx, sf_rest, alpha, beta, ply);
    }

    ctx.nodes += 1;
    if ctx.should_abort() {
        return alpha;
    }

    // Try transposition table
    let mut tt_move = Move::NULL;
    if let Some(entry) = ctx.tt.get(board.hash()) {
        tt_move = entry.best_move;
        if !is_pv && entry.depth >= depth {
            let tt_score = score_from_tt(entry.score, ply);
            match entry.flag {
                TTFlag::Exact => return tt_score,
                TTFlag::LowerBound if tt_score >= beta => return beta,
                TTFlag::UpperBound if tt_score <= alpha => return alpha,
                _ => {}
            }
            if alpha >= beta {
                return tt_score;
            }
        }
    }

    let in_check = board.is_in_check();

    // NMP
    // TODO: Add guards https://chessprogramming.org/Null_Move_Pruning
    if !is_pv && !is_null_move && !in_check && beta.abs() < MATE_THRESHOLD && depth >= 4 {
        let mut nmp_board = *board;
        let prev_boundary = ctx.rep.push_null(nmp_board.hash());
        let r = if depth >= 6 { 3 } else { 2 }; // Needs tuning
        nmp_board.swap_turn();
        let score = -negamax(
            &nmp_board,
            ctx,
            sf_rest,
            depth - 1 - r,
            -beta,
            -(beta - 1),
            ply + 1,
            true,
            false,
        );
        ctx.rep.pop_null(prev_boundary);
        if ctx.aborted {
            return alpha;
        }
        if score >= beta {
            return beta;
        }
    }

    // Generate legal moves
    board.generate_moves(moves);
    if moves.is_empty() {
        return if in_check {
            // Delay getting mated
            -MATE + ply as i32
        } else {
            // Stalemate
            0
        };
    };

    let orig_alpha = alpha;
    let mut best_move = Move::NULL;
    score_moves(scores, board, moves, tt_move, ctx, ply_idx);

    for i in 0..moves.len() {
        pick_move(moves, scores, i);
        let m = moves[i];

        let mut next_board = *board;
        next_board.apply_move(m);

        let do_lmr = depth >= 3
            && i >= 3
            && !in_check
            && !m.is_capture()
            && !m.is_promotion()
            && !next_board.is_in_check()
            && m != ctx.killers[ply_idx][0]
            && m != ctx.killers[ply_idx][1];

        let r = if do_lmr {
            lmr_reduction(depth, i).min(depth - 2)
        } else {
            0
        };

        // First move: full window with is_pv
        // Later moves: scout with null move pruning
        ctx.rep.push(next_board.hash());
        let mut score = -negamax(
            &next_board,
            ctx,
            sf_rest,
            depth - 1 - if i == 0 { 0 } else { r },
            if i == 0 { -beta } else { -alpha - 1 },
            -alpha,
            ply + 1,
            false,
            i == 0 && is_pv,
        );

        // Re-visit if LMR failed high or PVS beat alpha
        if i > 0 && score > alpha && !ctx.aborted && (r > 0 || (is_pv && score < beta)) {
            score = -negamax(
                &next_board,
                ctx,
                sf_rest,
                depth - 1,
                -beta,
                -alpha,
                ply + 1,
                false,
                is_pv,
            );
        }
        ctx.rep.pop();

        if ctx.aborted {
            return alpha;
        }

        if score >= beta {
            // Update killers and history
            let is_quiet = !m.is_capture() && !m.is_promotion();
            if is_quiet {
                if ctx.killers[ply_idx][0] != m {
                    ctx.killers[ply_idx][1] = ctx.killers[ply_idx][0];
                    ctx.killers[ply_idx][0] = m;
                }
                let bonus = (depth as i32) * (depth as i32);
                let hist = &mut ctx.history[board.turn() as usize][m.from().idx()][m.to().idx()];
                *hist += bonus - (*hist * bonus.abs()) / MAX_HISTORY;
            }

            ctx.tt.store(
                board.hash(),
                depth,
                score_to_tt(beta, ply),
                TTFlag::LowerBound,
                m,
            );
            return beta;
        }
        if score > alpha {
            alpha = score;
            best_move = m;
            ctx.update_pv(ply_idx, m);
        }
    }

    let flag = if alpha > orig_alpha {
        TTFlag::Exact
    } else {
        TTFlag::UpperBound
    };
    ctx.tt.store(
        board.hash(),
        depth,
        score_to_tt(alpha, ply),
        flag,
        best_move,
    );

    alpha
}

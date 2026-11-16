mod quiescence;
mod transposition;

use crate::board::Board;
use crate::chess::moves::MoveKind::EnPassant;
use crate::chess::moves::{Move, MoveList};
use crate::chess::piece::PieceKind;
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::search::quiescence::quiescence;
use crate::search::transposition::{TTFlag, TranspositionTable, score_from_tt, score_to_tt};

const INF: i32 = 1_000_000;
const MATE: i32 = 100_000;
const MAX_PLY: usize = 64;
const MATE_THRESHOLD: i32 = MATE - MAX_PLY as i32;

struct SearchContext {
    // Counts nodes as search termination metric or UCI stat reporting
    nodes: u64,
    // Can be set to true to abort search
    aborted: bool,
    // Triangular PV table
    // Prevents overwriting PV when backtracking
    pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pv_len: [usize; MAX_PLY],
    // Tries refutations of similar moves first in an attempt
    // to save analysis of other branches
    // https://chessprogramming.org/Killer_Heuristic
    killers: [[Move; 2]; MAX_PLY],
    // Tries moves that frequently cause cutoffs first
    // as they should generally be good moves
    // https://chessprogramming.org/History_Heuristic
    history: [[[i32; MAX_PLY]; MAX_PLY]; 2],
    tt: TranspositionTable,
}

impl SearchContext {
    fn new() -> SearchContext {
        Self {
            nodes: 0,
            aborted: false,
            pv_table: [[Move::NULL; MAX_PLY]; MAX_PLY],
            pv_len: [0; MAX_PLY],
            killers: [[Move::NULL; 2]; MAX_PLY],
            history: [[[0; MAX_PLY]; MAX_PLY]; 2],
            tt: TranspositionTable::new(1 << 20),
        }
    }

    fn update_pv(&mut self, ply: usize, m: Move) {
        let child_len = self.pv_len[ply + 1];
        self.pv_table[ply][0] = m;

        // Copy child PV from ply + 1 into ply + 0..child_len
        let (parent_row, child_row) = self.pv_table.split_at_mut(ply + 1);
        parent_row[ply][1..1 + child_len].copy_from_slice(&child_row[0][..child_len]);
        self.pv_len[ply] = 1 + child_len;
    }

    fn root_pv(&self) -> &[Move] {
        &self.pv_table[0][..self.pv_len[0]]
    }
}

/// For beta cutoff: better to analyse high-valued pieces first
#[inline(always)]
fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        Pawn => 100,
        Knight => 300,
        Bishop => 300,
        Rook => 500,
        Queen => 900,
        King => 10000,
    }
}

/// Higher-scoring moves are searched earlier
fn score_move(board: &Board, m: Move, tt_move: Move, ctx: &SearchContext, ply: usize) -> i32 {
    // Move equal to the one in transposition table
    if tt_move != Move::NULL && m == tt_move {
        return 1_000_000;
    }

    if m.is_capture() {
        let taken = if m.kind() == EnPassant {
            Pawn
        } else {
            board.get_piece(m.to()).unwrap().kind
        };
        let attacker = board.get_piece(m.from()).unwrap().kind;

        // Taking high-value piece with low-value piece is good
        // https://chessprogramming.org/MVV-LVA
        return 100_000 + 10 * piece_value(taken) - piece_value(attacker);
    }

    if m.is_promotion() {
        return 90_000;
    }

    // Quiet moves
    // Killer moves first
    // https://chessprogramming.org/Killer_Heuristic
    if ctx.killers[ply][0] == m {
        return 80_000;
    }
    if ctx.killers[ply][1] == m {
        return 70_000;
    }

    ctx.history[board.turn() as usize][m.from().idx()][m.to().idx()]
}

/// Generate move scores for all moves
fn score_moves(
    board: &Board,
    moves: &MoveList,
    tt_move: Move,
    ctx: &SearchContext,
    ply: usize,
) -> [i32; 256] {
    let mut scores = [0; 256];
    for (i, &m) in moves.iter().enumerate() {
        scores[i] = score_move(board, m, tt_move, ctx, ply);
    }
    scores
}

/// Swaps the highest-scoring move into `moves[idx]`
fn pick_move(moves: &mut MoveList, scores: &mut [i32; 256], idx: usize) {
    let mut best_idx = idx;
    let mut best_score = scores[idx];

    for j in (idx + 1)..moves.len() {
        if scores[j] > best_score {
            best_score = scores[j];
            best_idx = j;
        }
    }

    if best_idx != idx {
        moves.swap(idx, best_idx);
        scores.swap(best_idx, idx);
    }
}

/// https://chessprogramming.org/Negamax
pub fn best_move(board: &Board, depth: u8) -> Option<Move> {
    let moves = board.generate_moves();
    if moves.is_empty() {
        return None;
    }

    let mut best = moves[0];
    if depth == 0 {
        return Some(best);
    }

    let mut ctx = SearchContext::new();
    let mut alpha = -INF;
    let beta = INF;

    for m in moves {
        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(&next_board, &mut ctx, depth - 1, -beta, -alpha, 1);

        if score > alpha {
            alpha = score;
            best = m;
        }
    }

    Some(best)
}

/// https://chessprogramming.org/Negamax
fn negamax(
    board: &Board,
    ctx: &mut SearchContext,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    ply: u8,
) -> i32 {
    let ply_idx = ply as usize;
    ctx.pv_len[ply_idx] = 0; // Reset PV from prior branches

    if depth == 0 || ply_idx >= MAX_PLY - 1 {
        return quiescence(board, ctx, alpha, beta, ply);
    }

    ctx.nodes += 1;

    // Try transposition table
    let mut tt_move = Move::NULL;
    if let Some(entry) = ctx.tt.get(board.hash()) {
        tt_move = entry.best_move;
        if entry.depth >= depth {
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

    // Generate legal moves
    let mut moves = board.generate_moves();
    if moves.is_empty() {
        return if board.is_in_check() {
            // Delay getting mated
            -MATE + ply as i32
        } else {
            // Stalemate
            0
        };
    };

    let orig_alpha = alpha;
    let mut best_move = Move::NULL;
    let mut scores = score_moves(board, &moves, tt_move, ctx, ply_idx);

    for i in 0..moves.len() {
        pick_move(&mut moves, &mut scores, i);
        let m = moves[i];

        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(&next_board, ctx, depth - 1, -beta, -alpha, ply + 1);

        if ctx.aborted {
            return alpha;
        }

        if score >= beta {
            // Update killers and history
            if !m.is_capture() && ctx.killers[ply_idx][0] != m {
                ctx.killers[ply_idx][1] = ctx.killers[ply_idx][0];
                ctx.killers[ply_idx][0] = m;
            }
            if !m.is_capture() {
                ctx.history[board.turn() as usize][m.from().idx()][m.to().idx()] +=
                    (depth as i32) * (depth as i32);
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

pub fn search_infinite(
    board: &Board,
    on_info: impl Fn(u8, i32, u64, &[Move]), // depth, cp, nodes, best move
) -> Option<Move> {
    let mut moves = board.generate_moves();
    if moves.is_empty() {
        return None;
    }

    let mut best_move = moves[0];
    let mut ctx = SearchContext::new();

    for depth in 1..=MAX_PLY as u8 - 1 {
        let mut alpha = -INF;
        let beta = INF;
        let mut depth_best = best_move;
        ctx.pv_len[0] = 0;

        ctx.nodes += 1;
        let mut scores = score_moves(board, &moves, best_move, &ctx, 0);

        for i in 0..moves.len() {
            pick_move(&mut moves, &mut scores, i);
            let m = moves[i];

            let mut next_board = *board;
            next_board.apply_move(m);
            let score = -negamax(&next_board, &mut ctx, depth - 1, -beta, -alpha, 1);

            if ctx.aborted {
                break;
            }
            if score > alpha {
                alpha = score;
                depth_best = m;
                ctx.update_pv(0, m);
                on_info(depth, alpha, ctx.nodes, ctx.root_pv());
            }
        }

        if ctx.aborted {
            break;
        }

        best_move = depth_best;
        on_info(depth, alpha, ctx.nodes, ctx.root_pv());
    }

    Some(best_move)
}

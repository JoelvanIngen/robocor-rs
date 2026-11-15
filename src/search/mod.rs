use crate::board::Board;
use crate::chess::moves::Move;
use crate::chess::piece::PieceColour::White;
use crate::eval::eval_pos;

const INF: i32 = 1_000_000;
const MATE: i32 = 100_000;
const MAX_DEPTH: u8 = u8::MAX;
const MAX_PLY: usize = 16;

struct SearchContext {
    nodes: u64,
    aborted: bool,
    pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pv_len: [usize; MAX_PLY],
}

impl SearchContext {
    fn new() -> SearchContext {
        Self {
            nodes: 0,
            aborted: false,
            pv_table: [[Move::NULL; MAX_PLY]; MAX_PLY],
            pv_len: [0; MAX_PLY],
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

/// https://chessprogramming.org/Negamax
pub fn best_move(board: &Board, depth: u8) -> Option<Move> {
    let moves = board.generate_moves();
    if moves.is_empty() {
        return None;
    }

    let mut best = moves[0];
    let mut ctx = SearchContext::new();

    let mut alpha = -INF;
    let beta = INF;

    for m in moves {
        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(&next_board, &mut ctx, depth - 1, -beta, -alpha, 1);

        if score > alpha {
            alpha = score;
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

    ctx.nodes += moves.len() as u64;

    for m in moves {
        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(&next_board, ctx, depth - 1, -beta, -alpha, ply + 1);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
            ctx.update_pv(ply as usize, m);
        }

        if ctx.aborted {
            break;
        }
    }

    alpha
}

pub fn search_infinite(
    board: &Board,
    on_info: impl Fn(u8, i32, u64, &[Move]), // depth, cp, nodes, best move
) -> Option<Move> {
    let moves = board.generate_moves();
    if moves.is_empty() {
        return None;
    }

    let mut best_move = moves[0];
    let mut ctx = SearchContext::new();

    for depth in 1..=MAX_DEPTH {
        let mut alpha = -INF;
        let beta = INF;
        let mut depth_best = best_move;

        ctx.nodes += moves.len() as u64;

        for m in &moves {
            let mut next_board = *board;
            next_board.apply_move(*m);
            let score = -negamax(&next_board, &mut ctx, depth - 1, -beta, -alpha, 1);

            if ctx.aborted {
                break;
            }
            if score > alpha {
                alpha = score;
                depth_best = *m;
                ctx.update_pv(0, *m)
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

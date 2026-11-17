mod lmr;
mod negamax;
mod quiescence;
mod transposition;

use crate::board::Board;
use crate::chess::moves::MoveKind::EnPassant;
use crate::chess::moves::{Move, MoveList};
use crate::chess::piece::PieceKind;
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::engine::UciReport;
use crate::search::negamax::negamax;
use crate::search::transposition::TranspositionTable;

const INF: i32 = 1_000_000;
const MATE: i32 = 100_000;
pub const MAX_PLY: usize = 64;
const MATE_THRESHOLD: i32 = MATE - MAX_PLY as i32;
const MAX_HISTORY: i32 = 16_384;

/// Avoids having to repeatedly allocate and move data
struct SearchFrame {
    pub moves: MoveList,
    pub scores: [i32; 256],
}

impl SearchFrame {
    fn new() -> Self {
        Self {
            moves: MoveList::new(),
            scores: [0; 256],
        }
    }
}

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
    history: [[[i32; 64]; 64]; 2],
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
            tt: TranspositionTable::new(1 << 24),
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

fn update_report(
    report: &mut UciReport,
    depth: u8,
    score: i32,
    nodes: u64,
    hashfull: usize,
    pv: &[Move],
) {
    report.depth = depth as usize;
    report.score = score;
    report.nodes = nodes as usize;
    report.hashfull = hashfull;
    report.pv.clear();
    let _ = report.pv.try_extend_from_slice(pv);
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
    scores: &mut [i32; 256],
    board: &Board,
    moves: &MoveList,
    tt_move: Move,
    ctx: &SearchContext,
    ply: usize,
) {
    for (i, &m) in moves.iter().enumerate() {
        scores[i] = score_move(board, m, tt_move, ctx, ply);
    }
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
pub fn best_move(
    board: &Board,
    report: &mut UciReport,
    on_info: impl Fn(&UciReport),
    depth: u8,
) -> Option<Move> {
    let mut ctx = SearchContext::new();
    let mut search_frames: [SearchFrame; MAX_PLY] = std::array::from_fn(|_| SearchFrame::new());

    report.start_timer();

    let (sf, sf_rest) = search_frames.split_first_mut().unwrap();
    let moves = &mut sf.moves;
    let scores = &mut sf.scores;

    board.generate_moves(moves);
    if moves.is_empty() {
        return None;
    }

    let mut best = moves[0];
    if depth == 0 {
        return Some(best);
    }

    let mut alpha = -INF;
    let beta = INF;
    ctx.pv_len[0] = 0;
    ctx.nodes += 1;
    score_moves(scores, board, &moves, best, &ctx, 0);

    for i in 0..moves.len() {
        pick_move(moves, scores, i);
        let m = moves[i];

        let mut next_board = *board;
        next_board.apply_move(m);
        let score = -negamax(
            &next_board,
            &mut ctx,
            sf_rest,
            depth - 1,
            -beta,
            -alpha,
            1,
            false,
        );

        if ctx.aborted {
            break;
        }
        if score > alpha {
            alpha = score;
            best = m;
            ctx.update_pv(0, m);
            update_report(
                report,
                depth,
                alpha,
                ctx.nodes,
                ctx.tt.percent_full(),
                ctx.root_pv(),
            );
            on_info(report);
        }
    }

    Some(best)
}

pub fn search_infinite(
    board: &Board,
    report: &mut UciReport,
    on_info: impl Fn(&UciReport),
) -> Option<Move> {
    let mut ctx = SearchContext::new();
    let mut search_frames: [SearchFrame; MAX_PLY] = std::array::from_fn(|_| SearchFrame::new());

    report.start_timer();

    let (sf, sf_rest) = search_frames.split_first_mut().unwrap();
    let moves = &mut sf.moves;
    let scores = &mut sf.scores;

    board.generate_moves(moves);
    if moves.is_empty() {
        return None;
    }

    let mut best_move = moves[0];

    for depth in 1..=MAX_PLY as u8 - 1 {
        let mut alpha = -INF;
        let beta = INF;
        let mut depth_best = best_move;
        ctx.pv_len[0] = 0;

        ctx.nodes += 1;
        score_moves(scores, board, moves, best_move, &ctx, 0);

        for i in 0..moves.len() {
            pick_move(moves, scores, i);
            let m = moves[i];

            let mut next_board = *board;
            next_board.apply_move(m);
            let score = -negamax(
                &next_board,
                &mut ctx,
                sf_rest,
                depth - 1,
                -beta,
                -alpha,
                1,
                false,
            );

            if ctx.aborted {
                break;
            }
            if score > alpha {
                alpha = score;
                depth_best = m;
                ctx.update_pv(0, m);
                update_report(
                    report,
                    depth,
                    alpha,
                    ctx.nodes,
                    ctx.tt.percent_full(),
                    ctx.root_pv(),
                );
                on_info(report);
            }
        }

        if ctx.aborted {
            break;
        }

        best_move = depth_best;
        update_report(
            report,
            depth,
            alpha,
            ctx.nodes,
            ctx.tt.percent_full(),
            ctx.root_pv(),
        );
        on_info(report);
    }

    Some(best_move)
}

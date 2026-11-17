mod reporting;

use crate::board::{Board, Square};
use crate::chess::moves::MoveList;
use crate::chess::piece::PieceColour;
pub use crate::engine::reporting::UciReport;
use crate::error::FenError;
use crate::search::{MAX_PLY, TranspositionTable, search_infinite};
use crate::uci::Command::{Go, Position, Stop, UciNewGame};
use crate::uci::PositionInitType::{Fen, StartPos};
use crate::uci::{Command, GoOptions, PositionOptions};
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const DEFAULT_TT_SHIFT: usize = 24;
const MOVE_OVERHEAD_MS: usize = 50;

/// Stores search stuff such as the table, since spawning threads
/// requires static lifetimes and referencing Engine-owned data would
/// be very annoying
struct ActiveSearch {
    abort: Arc<AtomicBool>,
    handle: JoinHandle<TranspositionTable>,
}

pub struct SearchLimits {
    pub max_depth: u8,
    pub soft_deadline: Option<Instant>,
    pub hard_deadline: Option<Instant>,
    pub infinite: bool,
}

pub struct Engine {
    uci_tx: Sender<String>,
    uci_rx: Receiver<Command>,
    board: Option<Board>,
    tt: Option<TranspositionTable>,
    active_search: Option<ActiveSearch>,
}

impl Engine {
    pub fn new(uci_tx: Sender<String>, uci_rx: Receiver<Command>) -> Engine {
        Engine {
            uci_tx,
            uci_rx,
            board: None,
            tt: Some(TranspositionTable::new(1 << DEFAULT_TT_SHIFT)),
            active_search: None,
        }
    }

    pub fn start(&mut self) {
        while let Ok(cmd) = self.uci_rx.recv() {
            match cmd {
                UciNewGame => self.handle_new_game(),
                Go(opts) => self.handle_go(opts),
                Position(opts) => self.handle_position(opts),
                Stop => self.handle_stop(),
                cmd => {
                    panic!(
                        "engine received command from uci that shouldn't be passed to engine: {:?}",
                        cmd
                    )
                }
            }
        }
    }

    fn handle_new_game(&mut self) {
        if self.active_search.is_some() {
            self.handle_stop()
        }
        self.board = None;
        self.tt.as_mut().unwrap().clear();
    }

    fn handle_stop(&mut self) {
        if let Some(search) = self.active_search.take() {
            search.abort.store(true, Relaxed);
            self.tt = Some(search.handle.join().expect("search thread panicked"))
        }
        self.active_search = None;
    }

    fn handle_position(&mut self, opts: PositionOptions) {
        match opts.init_type {
            Fen(fen) => {
                self.board = Some(
                    Board::from_fen(fen.as_str()).unwrap_or_else(|e| self.handle_error(fen, e)),
                )
            }
            StartPos => self.board = Some(Board::start_position()),
        }

        if let Some(moves) = opts.moves {
            self.apply_moves(moves);
        }
    }

    fn apply_moves(&mut self, moves: Vec<String>) {
        let mut possible_moves = MoveList::new();
        let board = self.board.as_mut().unwrap();

        for m in &moves {
            let start_sq = Square::from_str(&m[0..2]).expect("invalid uci start square");
            let end_sq = Square::from_str(&m[2..4]).expect("invalid uci end square");
            let prom = if m.len() == 5 { Some(&m[4..5]) } else { None };

            board.generate_moves(&mut possible_moves);
            possible_moves.retain(|test_m| test_m.from() == start_sq && test_m.to() == end_sq);

            if let Some(p) = prom {
                possible_moves.retain(|test_m| test_m.promotion_piece_uci() == p);
            }

            debug_assert!(
                possible_moves.len() == 1,
                "ambiguous moves from uci or no legal moves possible, possible moves: {:?}",
                possible_moves
            );

            board.apply_move(possible_moves[0]);
        }
    }

    fn handle_error(&mut self, fen: String, e: FenError) -> ! {
        self.uci_tx
            .send(format!("invalid fen {fen}: {:?}", e))
            .expect("failed send fen error message from engine to uci");
        exit(1);
    }

    fn handle_go(&mut self, opts: GoOptions) {
        let board = match self.board {
            Some(board) => board,
            None => {
                self.uci_tx
                    .send(String::from("sent `go infinite`, but no board was set up"))
                    .expect("failed to send message from engine to uci");
                return;
            }
        };

        if self.active_search.is_some() {
            self.handle_stop()
        }

        let mut tt = self
            .tt
            .take()
            .expect("transposition table missing in engine");
        let abort = Arc::new(AtomicBool::new(false));
        let abort_clone = Arc::clone(&abort);
        let uci_tx = self.uci_tx.clone();

        let search_limits = compute_search_limits(&board, opts);

        let handle = thread::spawn(move || {
            let mut report = UciReport::new();
            let best_move = search_infinite(
                &board,
                &mut tt,
                &abort_clone,
                search_limits,
                &mut report,
                |report: &UciReport| {
                    uci_tx
                        .send(report.to_str())
                        .expect("failed to send analysis info from engine to uci")
                },
            );
            let move_str = best_move.map_or_else(|| String::from("a1a1"), |m| m.to_uci());
            uci_tx
                .send(format!("bestmove {move_str}"))
                .expect("failed to send bestmove from engine to uci");
            tt
        });

        self.active_search = Some(ActiveSearch { abort, handle })
    }
}

/// Compute how much time we want to spend on a move
fn compute_search_limits(board: &Board, opts: GoOptions) -> SearchLimits {
    let now = Instant::now();
    let max_depth = opts
        .depth
        .unwrap_or((MAX_PLY - 1) as u8)
        .min((MAX_PLY - 1) as u8);

    if opts.infinite {
        return SearchLimits {
            max_depth,
            soft_deadline: None,
            hard_deadline: None,
            infinite: true,
        };
    }

    if let Some(mt) = opts.movetime {
        // Leave some time for overhead
        let budget = mt.saturating_sub(MOVE_OVERHEAD_MS as u64).max(1);
        let deadline = Instant::now() + Duration::from_millis(budget);
        return SearchLimits {
            max_depth,
            soft_deadline: Some(deadline),
            hard_deadline: Some(deadline),
            infinite: false,
        };
    }

    let (time, inc) = match board.turn() {
        PieceColour::White => (opts.wtime, opts.winc.unwrap_or(0)),
        PieceColour::Black => (opts.btime, opts.binc.unwrap_or(0)),
    };

    if let Some(rem) = time {
        let usable = rem.saturating_sub(MOVE_OVERHEAD_MS as u64);

        // I made this up, needs tweaking
        let target = (usable / 5) + (inc / 2);
        // Hard limit: 1.5 times target, or half the remaining time
        let hard_ms = ((target * 3) / 2).min(usable / 2).max(1);
        // Soft limit: 0.8 times target
        let soft_ms = ((target * 8) / 10).min(hard_ms).max(1);

        return SearchLimits {
            max_depth,
            soft_deadline: Some(now + Duration::from_millis(soft_ms)),
            hard_deadline: Some(now + Duration::from_millis(hard_ms)),
            infinite: false,
        };
    }

    SearchLimits {
        max_depth,
        soft_deadline: None,
        hard_deadline: None,
        infinite: false,
    }
}

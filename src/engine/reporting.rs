//! Reporting of analysis progress to UCI

use crate::chess::moves::Move;
use arrayvec::ArrayVec;
use std::time::Instant;

pub struct UciReport {
    pub depth: usize,
    pub score: i32,
    pub nodes: usize,
    pub hashfull: usize,
    pub pv: ArrayVec<Move, 64>, // Retrieving from MAX_PLY throws a type error for some reason

    start_time: Instant,
}

impl UciReport {
    pub fn new() -> Self {
        Self {
            depth: 0,
            score: 0,
            nodes: 0,
            hashfull: 0,
            pv: ArrayVec::new(),

            start_time: Instant::now(),
        }
    }

    pub fn start_timer(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn to_str(&self) -> String {
        let pv_str = self
            .pv
            .iter()
            .map(Move::to_uci)
            .collect::<Vec<_>>()
            .join(" ");

        // Use microseconds to prevent fast moves reporting inf nps
        let nps = (self.nodes as u128 * 1_000_000) / (Instant::now() - self.start_time).as_micros();

        let mut s = String::from("info");
        s.push_str(format!(" depth {}", self.depth).as_str());
        s.push_str(format!(" score cp {}", self.score).as_str());
        s.push_str(format!(" nodes {}", self.nodes).as_str());
        s.push_str(format!(" nps {}", nps).as_str());
        s.push_str(format!(" hashfull {}", self.hashfull).as_str());
        s.push_str(format!(" pv {}", pv_str).as_str());
        s
    }
}

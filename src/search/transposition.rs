use crate::chess::moves::Move;
use crate::search::MATE_THRESHOLD;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TTFlag {
    Empty,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TTEntry {
    pub key: u64,
    pub best_move: Move,
    pub score: i32,
    pub depth: u8,
    pub flag: TTFlag,
}

impl TTEntry {
    const EMPTY: Self = Self {
        key: 0,
        best_move: Move::NULL,
        score: 0,
        depth: 0,
        flag: TTFlag::Empty,
    };
}

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TranspositionTable {
    /// size must be power of two to allow mask to work correctly
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![TTEntry::EMPTY; size],
            mask: size - 1,
        }
    }

    pub fn get(&self, hash: u64) -> Option<TTEntry> {
        let entry = self.entries[(hash as usize) & self.mask];
        // Not empty and not accidental collision
        if entry.flag != TTFlag::Empty && entry.key == hash {
            Some(entry)
        } else {
            None
        }
    }

    pub fn store(&mut self, hash: u64, depth: u8, score: i32, flag: TTFlag, best_move: Move) {
        let idx = (hash as usize) & self.mask;
        let existing = self.entries[idx];
        // Always replace if higher depth or empty
        if existing.flag == TTFlag::Empty || depth >= existing.depth || existing.key != hash {
            self.entries[idx] = TTEntry {
                key: hash,
                best_move,
                score,
                depth,
                flag,
            };
        }
    }
}

/// Convert between ply-independent scores and mate scores that take
/// distance-until-mate into account
pub fn score_to_tt(score: i32, ply: u8) -> i32 {
    if score > MATE_THRESHOLD {
        score + ply as i32
    } else if score < -MATE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}
pub fn score_from_tt(score: i32, ply: u8) -> i32 {
    if score > MATE_THRESHOLD {
        score - ply as i32
    } else if score < -MATE_THRESHOLD {
        score + ply as i32
    } else {
        score
    }
}

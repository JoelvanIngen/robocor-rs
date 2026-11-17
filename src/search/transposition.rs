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
    size: usize,
    stored: usize,
}

impl TranspositionTable {
    /// size must be power of two to allow mask to work correctly
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![TTEntry::EMPTY; size],
            mask: size - 1,
            size,
            stored: 0,
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

        if existing.flag == TTFlag::Empty {
            self.stored += 1;
            self.entries[idx] = TTEntry {
                key: hash,
                best_move,
                score,
                depth,
                flag,
            };
        }

        // Replace if higher depth or same position
        if depth >= existing.depth || existing.key == hash {
            let final_move = if existing.key == hash && best_move == Move::NULL {
                existing.best_move
            } else {
                best_move
            };
            self.entries[idx] = TTEntry {
                key: hash,
                best_move: final_move,
                score,
                depth,
                flag,
            };
        }
    }

    pub fn percent_full(&self) -> usize {
        (self.stored * 1000) / self.size
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

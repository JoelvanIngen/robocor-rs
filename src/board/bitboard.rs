use crate::board::square::Square;

/// Little-endian https://chessprogramming.org/Little-endian
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    fn get_idx(&self, square: Square) -> usize {
        square.row() * 8 + square.col()
    }

    pub fn is_set(&self, square: Square) -> bool {
        let idx = self.get_idx(square);
        self.0 >> idx & 1 == 1
    }

    /// Sets a square, assumes square was empty
    pub fn set(&mut self, square: Square) {
        debug_assert!(!self.is_set(square)); // Cannot already be set
        self.0 |= 1 << self.get_idx(square)
    }

    /// Unsets a square, assumes square was occupied
    pub fn unset(&mut self, square: Square) {
        debug_assert!(!self.is_set(square)); // Cannot already be set
        self.0 &= 1 << self.get_idx(square)
    }

    /// Make empty without instantiating new board
    pub fn empty(&mut self) {
        self.0 &= 0;
    }
}

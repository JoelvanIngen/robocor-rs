use crate::board::square::Square;

/// Little-endian https://chessprogramming.org/Little-endian
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

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

    pub fn iter(&self) -> BitBoardIter {
        BitBoardIter(self.0)
    }
}

pub struct BitBoardIter(u64);

impl Iterator for BitBoardIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let idx = self.0.trailing_zeros() as usize;
            self.0 &= self.0 - 1;
            Some(Square::from_index(idx))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.0.count_ones() as usize;
        (count, Some(count))
    }
}

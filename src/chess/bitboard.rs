use crate::board::Square;
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

pub const BOARD_DIVIDER: &str = "  +---+---+---+---+---+---+---+---+";

/// Little-endian https://chessprogramming.org/Little-endian
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    #[inline(always)]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    #[inline(always)]
    const fn get_idx(&self, square: Square) -> usize {
        square.row() * 8 + square.col()
    }

    #[inline(always)]
    pub const fn is_set(&self, square: Square) -> bool {
        self.is_set_idx(self.get_idx(square))
    }

    #[inline(always)]
    pub const fn is_set_idx(&self, idx: usize) -> bool {
        self.0 >> idx & 1 == 1
    }

    /// Sets a square, assumes square was empty
    #[inline(always)]
    pub const fn set(&mut self, square: Square) {
        self.set_idx(self.get_idx(square));
    }

    #[inline(always)]
    pub const fn set_idx(&mut self, idx: usize) {
        debug_assert!(!self.is_set_idx(idx)); // Cannot already be set
        self.0 |= 1 << idx
    }

    /// Unsets a square, assumes square was occupied
    #[inline(always)]
    pub fn unset(&mut self, square: Square) {
        self.unset_idx(self.get_idx(square))
    }

    #[inline(always)]
    pub fn unset_idx(&mut self, idx: usize) {
        debug_assert!(self.is_set_idx(idx)); // Must already be set
        self.0 -= 1 << idx
    }

    /// Retrieves square corresponding to the current value
    /// Can only be used if single bit is set
    #[inline(always)]
    pub fn get_square(&self) -> Square {
        debug_assert_eq!(self.count(), 1, "More than one square set in bitmap");
        Square::from_index(self.0.trailing_zeros() as usize)
    }

    #[inline(always)]
    pub fn iter(&self) -> BitBoardIter {
        BitBoardIter(self.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        BitBoard(!self.0)
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

impl FromIterator<Square> for BitBoard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        let mut bb = BitBoard::EMPTY;
        for square in iter {
            bb.set(square);
        }
        bb
    }
}

impl FromIterator<BitBoard> for BitBoard {
    fn from_iter<T: IntoIterator<Item = BitBoard>>(iter: T) -> Self {
        let mut bb = BitBoard::EMPTY;
        for other in iter {
            bb |= other;
        }
        bb
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{BOARD_DIVIDER}")?;
        for row_idx in (0..8).rev() {
            write!(f, "{} |", row_idx + 1)?;
            for col_idx in 0..8 {
                match self.is_set(Square::from_coords(row_idx, col_idx)) {
                    true => write!(f, " * |")?,
                    false => write!(f, "   |")?,
                };
            }
            writeln!(f)?;
            writeln!(f, "{BOARD_DIVIDER}")?;
        }

        writeln!(f, "    A   B   C   D   E   F   G   H")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// Two bits are set, we move one of them
    #[test]
    fn move_one() {
        let mut bb = BitBoard::EMPTY;
        bb.set(Square::from_str("b1").unwrap());
        assert_eq!(bb, BitBoard::from_u64(2));
        bb.set(Square::from_str("g1").unwrap());
        assert_eq!(bb, BitBoard::from_u64(66));

        bb.unset(Square::from_str("b1").unwrap());
        assert_eq!(bb, BitBoard::from_u64(64));

        bb.set(Square::from_str("a3").unwrap());
        assert_eq!(bb, BitBoard::from_u64(65600));
    }
}

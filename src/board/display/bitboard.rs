use crate::board::bitboard::BitBoard;
use crate::board::display::BOARD_DIVIDER;
use crate::board::square::Square;
use std::fmt::{Display, Formatter};

impl Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{BOARD_DIVIDER}")?;
        for row_idx in (0..8).rev() {
            write!(f, "{} |", row_idx)?;
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

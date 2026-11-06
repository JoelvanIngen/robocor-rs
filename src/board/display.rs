use crate::board::Board;
use crate::chess::square::Square;
use std::fmt::{Display, Formatter};

pub const BOARD_DIVIDER: &str = "  +---+---+---+---+---+---+---+---+";

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{BOARD_DIVIDER}")?;
        for row_idx in (0..8).rev() {
            write!(f, "{} |", row_idx + 1)?;
            for col_idx in 0..8 {
                match self.get_piece(Square::from_coords(row_idx, col_idx)) {
                    Some(piece) => write!(f, " {piece} |")?,
                    None => write!(f, "   |")?,
                }
            }
            writeln!(f)?;
            writeln!(f, "{BOARD_DIVIDER}")?;
        }

        writeln!(f, "    A   B   C   D   E   F   G   H")
    }
}

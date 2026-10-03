use crate::board::Board;
use crate::board::display::BOARD_DIVIDER;
use crate::board::square::Square;
use std::fmt::{Display, Formatter};

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{BOARD_DIVIDER}")?;
        for row_idx in (0..8).rev() {
            write!(f, "{} |", row_idx)?;
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

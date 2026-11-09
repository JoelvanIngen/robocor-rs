use crate::board::Board;
use crate::chess::piece::{Piece, PieceColour, PieceKind};
use std::fmt::{Display, Formatter};

pub const BOARD_DIVIDER: &str = "  +---+---+---+---+---+---+---+---+";

fn create_piece_grid(board: &Board) -> [[Option<Piece>; 8]; 8] {
    let mut grid = [[None; 8]; 8];

    for colour in PieceColour::ALL {
        for kind in PieceKind::ALL {
            for sq in board.bitboards[colour as usize][kind as usize].iter() {
                grid[sq.row()][sq.col()] = Some(Piece::new(kind, colour));
            }
        }
    }

    grid
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let grid = create_piece_grid(self);

        writeln!(f, "{BOARD_DIVIDER}")?;
        for row_idx in (0..8).rev() {
            write!(f, "{} |", row_idx + 1)?;
            for col_idx in 0..8 {
                match grid[row_idx][col_idx] {
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

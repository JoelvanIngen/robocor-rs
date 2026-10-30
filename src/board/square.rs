use crate::error::NotationError;
use crate::error::NotationError::{InvalidNotation, InvalidSquare};
use std::str::FromStr;

/// Pre-init all squares on the board for cheap iteration
pub const BOARD_SQUARES: [Square; 64] = {
    let mut squares = [Square::from_index(0); 64];
    let mut i = 0;
    while i < 64 {
        let square = Square::from_index(i);
        squares[i] = square;
        i += 1;
    }
    squares
};

/// row * 8 + col
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub const fn from_coords(row: usize, col: usize) -> Square {
        debug_assert!(row < 8 && col < 8, "Square out of bounds");
        Square::from_index(row * 8 + col)
    }

    pub const fn from_index(index: usize) -> Square {
        debug_assert!(index < 64, "Square out of bounds");
        Square(index as u8)
    }

    pub const fn row(&self) -> usize {
        (self.0 >> 3) as usize
    }

    pub const fn col(&self) -> usize {
        (self.0 & 0b00_000_111) as usize
    }

    pub fn idx(&self) -> usize {
        self.0 as usize
    }
}

impl FromStr for Square {
    type Err = NotationError;

    /// Converts chess square notation (a1-h8) to internal Square representation ((0,0)-(7,7))
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(InvalidNotation(s.to_string()));
        }

        let row_not = match s.chars().nth(0) {
            Some(c) => c,
            _ => return Err(InvalidNotation(s.to_string())),
        };

        let col_not = match s.chars().nth(1) {
            Some(c) => c,
            _ => return Err(InvalidNotation(s.to_string())),
        };

        let col = (u32::from(row_not) - u32::from('a')) as usize;
        let row = (u32::from(col_not) - u32::from('1')) as usize;

        if row > 7 || col > 7 {
            return Err(InvalidSquare(s.to_string()));
        }

        Ok(Square::from_coords(row, col))
    }
}

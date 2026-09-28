use crate::error::NotationError;
use crate::error::NotationError::{InvalidNotation, InvalidSquare};
use std::str::FromStr;

/// Bits 0-1: unused
/// Bits 2-5: row (1-8)
/// Bits 6-8: col (A-H)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub fn new(row: usize, col: usize) -> Square {
        Square((row as u8) << 3 | (col as u8))
    }

    pub fn row(&self) -> usize {
        (self.0 >> 3) as usize
    }

    pub fn col(&self) -> usize {
        (self.0 & 0b00_000_111) as usize
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

        Ok(Square::new(row, col))
    }
}

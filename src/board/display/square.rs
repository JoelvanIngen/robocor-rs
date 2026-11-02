use crate::board::Square;
use std::fmt::Display;

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_letter = (b'a' + self.col() as u8) as char;
        write!(f, "{}{}", col_letter, self.row() + 1)
    }
}

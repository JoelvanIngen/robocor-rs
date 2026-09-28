use crate::board::piece::PieceColour::{Black, White};
use crate::board::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::error::FenError;
use crate::error::FenError::InvalidFenPiece;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    pub const COUNT: usize = 6;
}

impl TryFrom<char> for PieceKind {
    type Error = FenError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'k' => Ok(King),
            'q' => Ok(Queen),
            'r' => Ok(Rook),
            'b' => Ok(Bishop),
            'n' => Ok(Knight),
            'p' => Ok(Pawn),
            _ => Err(InvalidFenPiece(c)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceColour {
    White,
    Black,
}

impl PieceColour {
    pub const COUNT: usize = 2;
}

impl TryFrom<char> for PieceColour {
    type Error = FenError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.is_ascii_uppercase() {
            true => Ok(White),
            false => Ok(Black),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub colour: PieceColour,
}

impl TryFrom<char> for Piece {
    type Error = FenError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        Ok(Piece {
            kind: PieceKind::try_from(c)?,
            colour: PieceColour::try_from(c)?,
        })
    }
}

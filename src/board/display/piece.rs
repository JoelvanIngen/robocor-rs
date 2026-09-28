use crate::board::piece::{Piece, PieceColour, PieceKind};
use std::fmt::{Display, Formatter};

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let piece_char = match self.kind {
            PieceKind::King => 'K',
            PieceKind::Queen => 'Q',
            PieceKind::Rook => 'R',
            PieceKind::Bishop => 'B',
            PieceKind::Knight => 'N',
            PieceKind::Pawn => 'P',
        };

        write!(
            f,
            "{}",
            match self.colour {
                PieceColour::White => piece_char,
                PieceColour::Black => piece_char.to_ascii_lowercase(),
            }
        )
    }
}

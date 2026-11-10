use crate::board::Board;
use crate::chess::piece::{PieceColour, PieceKind};

const QUEEN_VAL: f32 = 9.;
const ROOK_VAL: f32 = 5.;
const BISHOP_VAL: f32 = 3.;
const KNIGHT_VAL: f32 = 3.;
const PAWN_VAL: f32 = 1.;

/// Counts values of all pieces on the board and returns
/// difference in favour of white
/// Naive implementation; needs optimisation later
pub fn piece_values(board: &Board) -> f32 {
    let mut eval = 0f32;

    for piece in board.get_pieces() {
        let value = match piece.kind {
            PieceKind::Pawn => PAWN_VAL,
            PieceKind::Knight => KNIGHT_VAL,
            PieceKind::Bishop => BISHOP_VAL,
            PieceKind::Rook => ROOK_VAL,
            PieceKind::Queen => QUEEN_VAL,
            _ => continue,
        };

        eval += match piece.colour {
            PieceColour::White => value,
            PieceColour::Black => -value,
        }
    }

    eval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_board() {
        let board = Board::start_position();
        assert_eq!(
            piece_values(&board),
            0.0,
            "piece value difference of start position should be zero"
        );
    }

    #[test]
    fn test_missing_black_queen() {
        let board =
            Board::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            9.0,
            "queen difference should be 9.0 in favour of white"
        );
    }

    #[test]
    fn test_missing_white_bishop() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RN1QKBNR w KQkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            -3.0,
            "bishop difference should be 3.0 in favour of black"
        )
    }

    #[test]
    fn test_missing_black_knight() {
        let board =
            Board::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            3.0,
            "knight difference should be 3.0 in favour of white"
        )
    }

    #[test]
    fn test_missing_white_rook() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            -5.0,
            "rook difference should be 5.0 in favour of black"
        );
    }
}

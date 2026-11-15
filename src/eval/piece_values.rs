use crate::board::Board;
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::{Bishop, Knight, Pawn, Queen, Rook};
use crate::chess::piece::Piece;

const QUEEN_VAL: i32 = 900;
const ROOK_VAL: i32 = 500;
const BISHOP_VAL: i32 = 300;
const KNIGHT_VAL: i32 = 300;
const PAWN_VAL: i32 = 100;

/// Counts values of all pieces on the board and returns
/// difference in favour of white
/// Naive implementation; needs optimisation later
pub fn piece_values(board: &Board) -> i32 {
    let mut eval = 0;

    eval += board.get_bitboard(Piece::new(Queen, White)).count() as i32 * QUEEN_VAL;
    eval -= board.get_bitboard(Piece::new(Queen, Black)).count() as i32 * QUEEN_VAL;
    eval += board.get_bitboard(Piece::new(Rook, White)).count() as i32 * ROOK_VAL;
    eval -= board.get_bitboard(Piece::new(Rook, Black)).count() as i32 * ROOK_VAL;
    eval += board.get_bitboard(Piece::new(Bishop, White)).count() as i32 * BISHOP_VAL;
    eval -= board.get_bitboard(Piece::new(Bishop, Black)).count() as i32 * BISHOP_VAL;
    eval += board.get_bitboard(Piece::new(Knight, White)).count() as i32 * KNIGHT_VAL;
    eval -= board.get_bitboard(Piece::new(Knight, Black)).count() as i32 * KNIGHT_VAL;
    eval += board.get_bitboard(Piece::new(Pawn, White)).count() as i32 * PAWN_VAL;
    eval -= board.get_bitboard(Piece::new(Pawn, Black)).count() as i32 * PAWN_VAL;

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
            0,
            "piece value difference of start position should be zero"
        );
    }

    #[test]
    fn test_missing_black_queen() {
        let board =
            Board::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            900,
            "queen difference should be 9.0 in favour of white"
        );
    }

    #[test]
    fn test_missing_white_bishop() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RN1QKBNR w KQkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            -300,
            "bishop difference should be 3.0 in favour of black"
        )
    }

    #[test]
    fn test_missing_black_knight() {
        let board =
            Board::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            300,
            "knight difference should be 3.0 in favour of white"
        )
    }

    #[test]
    fn test_missing_white_rook() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1").unwrap();
        assert_eq!(
            piece_values(&board),
            -500,
            "rook difference should be 5.0 in favour of black"
        );
    }
}

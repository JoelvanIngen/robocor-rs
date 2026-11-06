use crate::board::{BitBoard, Board};
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::chess::piece::{Piece, PieceColour};
use crate::move_gen::{
    get_attacks_bishop, get_attacks_king, get_attacks_knight, get_attacks_rook, is_square_attacked,
};

impl Board {
    /// Places hypothetical pieces on the target square(s) to see if they can see
    /// opponent pieces of the same type, as that implies the position is attacked
    /// colour: colour of friendly
    /// target_bb: bitboard of squares to check
    /// TODO: Refactor such that this function receives a single square
    ///       as some unnecessary circular conversion is happening right now
    /// TODO: Skip piece check if that opponent's piece is no longer present on the board
    /// TODO: We might be able to check queen during rooks/bishops to save on double computation
    pub(super) fn is_bb_attacked(&self, colour: PieceColour, target_bb: BitBoard) -> bool {
        let enemy_colour = colour.other();

        let friendly = self.get_colour_bitboard(colour);
        let enemy = self.get_colour_bitboard(colour.other());

        let enemy_queens = self.get_bitboard(Piece::new(Queen, enemy_colour));
        let enemy_rooks = self.get_bitboard(Piece::new(Rook, enemy_colour));
        let enemy_bishops = self.get_bitboard(Piece::new(Bishop, enemy_colour));
        let enemy_knights = self.get_bitboard(Piece::new(Knight, enemy_colour));
        let enemy_pawns = self.get_bitboard(Piece::new(Pawn, enemy_colour));

        let has_enemy_queens = enemy_queens != BitBoard::EMPTY;
        let has_enemy_rooks = enemy_rooks != BitBoard::EMPTY;
        let has_enemy_bishops = enemy_bishops != BitBoard::EMPTY;
        let has_enemy_knights = enemy_knights != BitBoard::EMPTY;
        let has_enemy_pawns = enemy_pawns != BitBoard::EMPTY;

        let has_enemy_orthogonal = has_enemy_queens || has_enemy_rooks;
        let has_enemy_diagonal = has_enemy_queens || has_enemy_bishops;

        let enemy_orthogonal_movers = enemy_queens | enemy_rooks;
        let enemy_diagonal_movers = enemy_queens | enemy_bishops;

        // King
        if get_attacks_king(
            self.get_bitboard(Piece::new(King, enemy_colour))
                .get_square(),
            BitBoard::EMPTY,
        ) & target_bb
            != BitBoard::EMPTY
        {
            return true;
        }

        // Other pieces
        for sq in target_bb.iter() {
            if has_enemy_orthogonal
                && (get_attacks_rook(sq, friendly, enemy) & enemy_orthogonal_movers
                    != BitBoard::EMPTY)
            {
                return true;
            }
            if has_enemy_diagonal
                && (get_attacks_bishop(sq, friendly, enemy) & enemy_diagonal_movers
                    != BitBoard::EMPTY)
            {
                return true;
            }
            // TODO: The friendly bitboard is irrelevant here, should refactor so we can omit it
            if has_enemy_knights
                && (get_attacks_knight(sq, BitBoard::EMPTY) & enemy_knights != BitBoard::EMPTY)
            {
                return true;
            }
            if has_enemy_pawns && is_square_attacked(sq, enemy_pawns, colour) {
                return true;
            }
        }

        false
    }
}

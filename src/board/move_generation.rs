use crate::board::{Board, get_pl_moves_knight};
use crate::chess::moves::MoveList;
use crate::chess::piece::PieceColour;
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::move_gen::{
    get_pl_moves_bishop, get_pl_moves_king, get_pl_moves_pawn, get_pl_moves_queen,
    get_pl_moves_rook,
};

impl Board {
    /// Generate all pseudo-legal moves for a given colour
    pub fn generate_pl_moves(&self, colour: PieceColour) -> MoveList {
        let mut moves = MoveList::new();

        let piece_bitboard = self.bitboards[colour as usize];
        let (friendly, enemy) = match colour {
            White => (
                self.get_colour_bitboard(White),
                self.get_colour_bitboard(Black),
            ),
            Black => (
                self.get_colour_bitboard(Black),
                self.get_colour_bitboard(White),
            ),
        };

        for pos in piece_bitboard[Bishop as usize].iter() {
            get_pl_moves_bishop(&mut moves, pos, friendly, enemy)
        }

        for pos in piece_bitboard[King as usize].iter() {
            get_pl_moves_king(
                &mut moves,
                pos,
                friendly,
                enemy,
                self.get_castling_rights(colour),
            )
        }

        for pos in piece_bitboard[Knight as usize].iter() {
            get_pl_moves_knight(&mut moves, pos, friendly, enemy)
        }

        for pos in piece_bitboard[Pawn as usize].iter() {
            get_pl_moves_pawn(
                &mut moves,
                pos,
                friendly,
                enemy,
                colour,
                self.en_passant_square(),
            )
        }

        for pos in piece_bitboard[Queen as usize].iter() {
            get_pl_moves_queen(&mut moves, pos, friendly, enemy)
        }

        for pos in piece_bitboard[Rook as usize].iter() {
            get_pl_moves_rook(&mut moves, pos, friendly, enemy)
        }

        moves
    }

    /// Shortcut to generate legal moves for the current colour
    pub fn generate_moves(&self) -> MoveList {
        self.generate_moves_colour(self.turn)
    }

    /// Generate all legal moves for a given colour
    /// Checks for legality
    /// Whilst only the colour whose turn it is, is allowed to move,
    /// analysis might require finding potential moves in absence
    /// of a move (threat finding), so takes colour explicitly
    pub fn generate_moves_colour(&self, colour: PieceColour) -> MoveList {
        let ctx = self.check_context(colour);
        let mut moves = MoveList::new();

        for pl_move in self.generate_pl_moves(colour) {
            if self.is_legal(pl_move, colour, &ctx) {
                moves.push(pl_move);
            }
        }

        moves
    }
}

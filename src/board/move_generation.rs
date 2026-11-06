use crate::board::{BitBoard, Board, get_pl_moves_knight};
use crate::chess::moves::MoveKind::{KingCastle, QueenCastle};
use crate::chess::moves::{Move, MoveKind, MoveList};
use crate::chess::piece::PieceColour::{Black, White};
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::chess::piece::{Piece, PieceColour};
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
        let mut moves = MoveList::new();

        for pl_move in self.generate_pl_moves(colour) {
            let mut new_board = *self;
            new_board.apply_move(pl_move);
            if !new_board.is_in_check(colour, pl_move) {
                moves.push(pl_move);
            }
        }

        moves
    }

    /// Checks if the king of a provided colour is in check by the opponent
    fn is_in_check(&self, colour: PieceColour, pl_move: Move) -> bool {
        let pl_move_kind = pl_move.kind();
        let check_bb = if pl_move_kind == KingCastle || pl_move_kind == QueenCastle {
            // If the king wants to castle, it cannot move through or from a check
            // (the relevant rook can move through attacked squares)
            self.get_castling_bb(colour, pl_move_kind)
        } else {
            self.get_bitboard(Piece::new(King, colour))
        };

        self.is_bb_attacked(colour, check_bb)
    }

    fn get_castling_bb(&self, colour: PieceColour, kind: MoveKind) -> BitBoard {
        match (colour, kind) {
            (White, QueenCastle) => BitBoard::from_u64(28),
            (White, KingCastle) => BitBoard::from_u64(112),
            (Black, QueenCastle) => BitBoard::from_u64(2017612633061982208),
            (Black, KingCastle) => BitBoard::from_u64(8070450532247928832),
            _ => unreachable!(),
        }
    }
}

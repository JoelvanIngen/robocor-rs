use crate::board::move_gen::{
    get_attacks_bishop, get_attacks_king, get_attacks_knight, get_attacks_pawn, get_attacks_queen,
    get_attacks_rook, get_pl_moves_bishop, get_pl_moves_king, get_pl_moves_pawn,
    get_pl_moves_queen, get_pl_moves_rook,
};
use crate::board::moves::MoveKind::{KingCastle, QueenCastle};
use crate::board::moves::{Move, MoveKind, MoveList};
use crate::board::piece::PieceColour::{Black, White};
use crate::board::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::board::piece::{Piece, PieceColour};
use crate::board::square::BOARD_SQUARES;
use crate::board::{BitBoard, Board, get_pl_moves_knight};

impl Board {
    /// Generate all pseudo-legal moves for a given colour
    pub fn generate_pl_moves(&self, colour: PieceColour) -> MoveList {
        let mut moves = MoveList::new();

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

        for pos in BOARD_SQUARES {
            let piece = match self.get_piece(pos) {
                Some(p) => p,
                None => continue,
            };
            if piece.colour != colour {
                continue;
            }
            match piece.kind {
                Bishop => get_pl_moves_bishop(&mut moves, pos, friendly, enemy),
                King => get_pl_moves_king(
                    &mut moves,
                    pos,
                    friendly,
                    enemy,
                    self.get_castling_rights(colour),
                ),
                Knight => get_pl_moves_knight(&mut moves, pos, friendly, enemy),
                Pawn => get_pl_moves_pawn(
                    &mut moves,
                    pos,
                    friendly,
                    enemy,
                    colour,
                    self.en_passant_square(),
                ),
                Queen => get_pl_moves_queen(&mut moves, pos, friendly, enemy),
                Rook => get_pl_moves_rook(&mut moves, pos, friendly, enemy),
            }
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
        let check_bb = if pl_move.kind == KingCastle || pl_move.kind == QueenCastle {
            // If the king wants to castle, it cannot move through or from a check
            // (the relevant rook can move through attacked squares)
            self.get_castling_bb(colour, pl_move.kind)
        } else {
            self.get_bitboard(Piece::new(King, colour))
        };

        self.is_bb_attacked(colour.other(), check_bb)
    }

    /// colour: colour of attacker (not the colour that made the move)
    /// target_bb: bitboard of squares to check
    pub(super) fn is_bb_attacked(&self, colour: PieceColour, target_bb: BitBoard) -> bool {
        let friendly = self.get_colour_bitboard(colour);
        let enemy = self.get_colour_bitboard(colour.other());

        // King
        if target_bb
            & self
                .get_bitboard(Piece::new(King, colour))
                .iter()
                .map(|s| get_attacks_king(s, friendly))
                .collect()
            != BitBoard::EMPTY
        {
            return true;
        };

        // Queen
        if target_bb
            & self
                .get_bitboard(Piece::new(Queen, colour))
                .iter()
                .map(|s| get_attacks_queen(s, friendly, enemy))
                .collect()
            != BitBoard::EMPTY
        {
            return true;
        };

        // Rooks
        if target_bb
            & self
                .get_bitboard(Piece::new(Rook, colour))
                .iter()
                .map(|s| get_attacks_rook(s, friendly, enemy))
                .collect()
            != BitBoard::EMPTY
        {
            return true;
        };

        // Bishops
        if target_bb
            & self
                .get_bitboard(Piece::new(Bishop, colour))
                .iter()
                .map(|s| get_attacks_bishop(s, friendly, enemy))
                .collect()
            != BitBoard::EMPTY
        {
            return true;
        };

        // Knights
        if target_bb
            & self
                .get_bitboard(Piece::new(Knight, colour))
                .iter()
                .map(|s| get_attacks_knight(s, friendly))
                .collect()
            != BitBoard::EMPTY
        {
            return true;
        };

        // Pawns
        if target_bb
            & self
                .get_bitboard(Piece::new(Pawn, colour))
                .iter()
                .map(|s| get_attacks_pawn(s, friendly, colour))
                .collect()
            != BitBoard::EMPTY
        {
            return true;
        };

        false
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

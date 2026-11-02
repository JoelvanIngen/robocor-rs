use crate::board::move_gen::{
    get_attacks_bishop, get_attacks_king, get_attacks_knight, get_attacks_pawn, get_attacks_queen,
    get_attacks_rook, get_pl_moves_bishop, get_pl_moves_king, get_pl_moves_pawn,
    get_pl_moves_queen, get_pl_moves_rook,
};
use crate::board::moves::MoveKind::{KingCastle, QueenCastle};
use crate::board::moves::{Move, MoveKind};
use crate::board::piece::PieceColour::{Black, White};
use crate::board::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::board::piece::{Piece, PieceColour};
use crate::board::square::BOARD_SQUARES;
use crate::board::{BitBoard, Board, Square, get_pl_moves_knight};

impl Board {
    /// Generate all pseudo-legal moves for a given colour
    pub fn generate_pl_moves(&self, colour: PieceColour) -> Vec<Move> {
        let mut moves = vec![];

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
            moves.extend(match piece.kind {
                Bishop => get_pl_moves_bishop(pos, friendly, enemy),
                King => get_pl_moves_king(pos, friendly, enemy, self.get_castling_rights(colour)),
                Knight => get_pl_moves_knight(pos, friendly, enemy),
                Pawn => get_pl_moves_pawn(pos, friendly, enemy, colour, self.en_passant_square()),
                Queen => get_pl_moves_queen(pos, friendly, enemy),
                Rook => get_pl_moves_rook(pos, friendly, enemy),
            })
        }

        moves
    }

    /// Shortcut to generate legal moves for the current colour
    pub fn generate_moves(&self) -> Vec<Move> {
        self.generate_moves_colour(self.turn)
    }

    /// Generate all legal moves for a given colour
    /// Checks for legality
    /// Whilst only the colour whose turn it is, is allowed to move,
    /// analysis might require finding potential moves in absence
    /// of a move (threat finding), so takes colour explicitly
    pub fn generate_moves_colour(&self, colour: PieceColour) -> Vec<Move> {
        let mut moves = vec![];

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
        let attacked = self.create_attacked_bitboard(colour.other());

        if pl_move.kind == KingCastle || pl_move.kind == QueenCastle {
            // If the king wants to castle, it cannot move through or from a check
            // (the relevant rook can move through attacked squares)
            self.check_castle_squares(colour, attacked, pl_move.kind)
        } else {
            self.get_bitboard(Piece::new(King, colour)) & attacked != BitBoard::EMPTY
        }
    }

    /// colour: colour of attacker (not the colour that made the move)
    pub(crate) fn create_attacked_bitboard(&self, colour: PieceColour) -> BitBoard {
        let mut bb = BitBoard::EMPTY;

        let friendly = self.get_colour_bitboard(colour);
        let enemy = self.get_colour_bitboard(colour.other());

        // King
        bb |= self
            .get_bitboard(Piece::new(King, colour))
            .iter()
            .map(|s| get_attacks_king(s, friendly))
            .collect();

        // Queen
        bb |= self
            .get_bitboard(Piece::new(Queen, colour))
            .iter()
            .map(|s| get_attacks_queen(s, friendly, enemy))
            .collect();

        // Rooks
        bb |= self
            .get_bitboard(Piece::new(Rook, colour))
            .iter()
            .map(|s| get_attacks_rook(s, friendly, enemy))
            .collect();

        // Bishops
        bb |= self
            .get_bitboard(Piece::new(Bishop, colour))
            .iter()
            .map(|s| get_attacks_bishop(s, friendly, enemy))
            .collect();

        // Knights
        bb |= self
            .get_bitboard(Piece::new(Knight, colour))
            .iter()
            .map(|s| get_attacks_knight(s, friendly))
            .collect();

        // Pawns
        bb |= self
            .get_bitboard(Piece::new(Pawn, colour))
            .iter()
            .map(|s| get_attacks_pawn(s, friendly, colour))
            .collect();

        bb
    }

    /// Check castling squares for checks, including the initial and final
    /// positions of the king
    fn check_castle_squares(
        &self,
        colour: PieceColour,
        attacked: BitBoard,
        castling_kind: MoveKind,
    ) -> bool {
        let row = match colour {
            White => 0,
            Black => 7,
        };
        let cols = match castling_kind {
            KingCastle => [2, 3, 4],
            QueenCastle => [4, 5, 6],
            _ => {
                panic!(
                    "Checking attacked castling squares but received non-castling move: {:?}",
                    castling_kind
                )
            }
        };

        cols.iter()
            .all(|col| !attacked.is_set(Square::from_coords(row, *col)))
    }
}

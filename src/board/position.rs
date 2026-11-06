use crate::board::position::GameState::{Draw, Ongoing, Won};
use crate::board::{Board, combine_bitboards};
use crate::chess::bitboard::BitBoard;
use crate::chess::moves::Move;
use crate::chess::moves::MoveKind::{
    BishopPromotion, BishopPromotionCapture, DoublePawnPush, EnPassant, KingCastle,
    KnightPromotion, KnightPromotionCapture, QueenCastle, QueenPromotion, QueenPromotionCapture,
    RookPromotion, RookPromotionCapture,
};
use crate::chess::piece::PieceColour::Black;
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Queen, Rook};
use crate::chess::piece::{Piece, PieceColour};
use crate::chess::square::Square;
use crate::move_gen::CASTLING_MASK;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Ongoing,
    Won(PieceColour),
    Draw,
}

impl Board {
    fn set_piece(&mut self, piece: Piece, square: Square) {
        self.bitboards[piece.colour as usize][piece.kind as usize].set(square);
        self.piece_at[square.row()][square.col()] = Some(piece);
        self.colour_bitboards[piece.colour as usize].set(square);
    }

    /// Removes a piece from both the BitBoard and the `piece_set` array,
    /// and ensures they stay in sync
    /// Piece must already be set
    fn unset_piece(&mut self, square: Square) -> Piece {
        let piece = self.piece_at[square.row()][square.col()]
            .expect("trying to unset a piece from an empty square");
        self.bitboards[piece.colour as usize][piece.kind as usize].unset(square);
        self.piece_at[square.row()][square.col()] = None;
        self.colour_bitboards[piece.colour as usize].unset(square);
        piece
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.piece_at[square.row()][square.col()]
    }

    pub fn apply_move(&mut self, m: Move) {
        debug_assert!(self.get_piece(m.from()).is_some(), "Moving piece not found");

        let kind = m.kind();
        let from = m.from();
        let to = m.to();

        // Captures
        if kind == EnPassant {
            // En-passant-ed piece must be on the starting row of the capturing piece
            let captured_row = from.row();
            self.unset_piece(Square::from_coords(captured_row, to.col()));
        } else if m.is_capture() {
            self.unset_piece(to);
        }

        // Move piece and handle promotion
        let moving_piece = self.unset_piece(from);
        let final_piece = if m.is_promotion() {
            let kind = match kind {
                QueenPromotionCapture | QueenPromotion => Queen,
                RookPromotionCapture | RookPromotion => Rook,
                BishopPromotionCapture | BishopPromotion => Bishop,
                KnightPromotionCapture | KnightPromotion => Knight,
                _ => unreachable!(),
            };
            Piece::new(kind, self.turn)
        } else {
            moving_piece
        };
        self.set_piece(final_piece, to);

        // Castling rook movement
        // King has already moved
        if kind == QueenCastle {
            let row = from.row();
            let rook = self.unset_piece(Square::from_coords(row, 0));
            self.set_piece(rook, Square::from_coords(row, 3));
        } else if kind == KingCastle {
            let row = from.row();
            let rook = self.unset_piece(Square::from_coords(row, 7));
            self.set_piece(rook, Square::from_coords(row, 5));
        }

        // Update castling rights
        self.castling_rights &= CASTLING_MASK[from.idx()];
        self.castling_rights &= CASTLING_MASK[to.idx()];

        self.en_passant = BitBoard::EMPTY;
        if kind == DoublePawnPush {
            // Get average of start and finish row, seems hacky but is
            // a shorter (and branchless) implementation rather than switching on
            // colour and performing +/- 1 arithmetic
            let ep_row = (from.row() + to.row()) / 2;
            self.en_passant.set(Square::from_coords(ep_row, to.col()));
        }

        // Update clocks
        self.halfmove_clock = match m.is_irreversable() {
            true => 0,
            false => self.halfmove_clock + 1,
        };
        if self.turn == Black {
            self.fullmove_number += 1;
        }

        // Update turn
        self.turn = self.turn.other();
    }

    pub fn game_state(&self) -> GameState {
        if self.halfmove_clock > 50 {
            return Draw;
        }
        if (combine_bitboards(self.bitboards[0]) & combine_bitboards(self.bitboards[1])).count()
            == 2
        {
            // Only kings remain
            return Draw;
        }
        let legal_moves = self.generate_moves();
        if self.is_bb_attacked(self.turn, self.get_bitboard(Piece::new(King, self.turn)))
            && legal_moves.is_empty()
        {
            // Checkmate
            return Won(self.turn.other());
        }
        if legal_moves.is_empty() {
            // Stalemate
            return Draw;
        }

        Ongoing
    }

    pub fn en_passant_square(&self) -> Option<Square> {
        match self.en_passant {
            BitBoard::EMPTY => None,
            bb => Some(bb.get_square()),
        }
    }

    pub fn get_colour_bitboard(&self, colour: PieceColour) -> BitBoard {
        self.colour_bitboards[colour as usize]
    }

    pub fn get_bitboard(&self, piece: Piece) -> BitBoard {
        self.bitboards[piece.colour as usize][piece.kind as usize]
    }

    fn get_mut_bitboard(&mut self, piece: Piece) -> &mut BitBoard {
        &mut self.bitboards[piece.colour as usize][piece.kind as usize]
    }

    pub fn get_castling_rights(&self, colour: PieceColour) -> Vec<Square> {
        let mut options = Vec::with_capacity(2);
        match colour {
            PieceColour::White => {
                let queenside = Square::from_coords(0, 0);
                let kingside = Square::from_coords(0, 7);
                if self.castling_rights.is_set(queenside) {
                    options.push(queenside);
                }
                if self.castling_rights.is_set(kingside) {
                    options.push(kingside);
                }
            }
            PieceColour::Black => {
                let queenside = Square::from_coords(7, 0);
                let kingside = Square::from_coords(7, 7);
                if self.castling_rights.is_set(queenside) {
                    options.push(queenside);
                }
                if self.castling_rights.is_set(kingside) {
                    options.push(kingside);
                }
            }
        }
        options
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    // New board returns 32 pieces
    #[test]
    fn count_pieces_new_board() {
        let board = Board::start_position();
        assert_eq!(
            (board.colour_bitboards[0] | board.colour_bitboards[1]).count(),
            32
        );
    }
}

use crate::board::bitboard::BitBoard;
use crate::board::moves::Move;
use crate::board::moves::MoveKind::{KingCastle, QueenCastle};
use crate::board::piece::{Piece, PieceColour, PieceKind};
use crate::board::square::Square;
use crate::error::FenError;
use crate::error::FenError::{
    InvalidBoardSetup, InvalidCastlingRights, InvalidFullmoveNumber, InvalidHalfmoveClock,
    InvalidTurn, MissingField,
};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    // 2D array bitboard[colour][piece]
    bitboards: [[BitBoard; PieceKind::COUNT]; PieceColour::COUNT],
    piece_at: [[Option<Piece>; 8]; 8],
    turn: PieceColour,
    castling_rights: BitBoard,
    en_passant: BitBoard,
    halfmove_clock: u8, // Max value 50
    fullmove_number: u16,
}

impl Board {
    pub fn start_position() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("invalid fen")
    }

    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let mut bitboards = [[BitBoard::EMPTY; PieceKind::COUNT]; PieceColour::COUNT];
        let mut piece_at: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        let mut fen_parts = fen.split_whitespace();

        // Pieces
        let fen_pieces = fen_parts.next().ok_or(MissingField(fen.to_string()))?;
        let rows = fen_pieces.split('/');
        let mut row_idx: usize = 8;
        for row in rows {
            row_idx -= 1;
            let mut col_idx: usize = 0;

            for char in row.chars() {
                if char.is_ascii_digit() {
                    col_idx += char.to_digit(10).unwrap() as usize;
                } else {
                    let piece = Piece::try_from(char)?;
                    bitboards[piece.colour as usize][piece.kind as usize]
                        .set(Square::from_coords(row_idx, col_idx));
                    piece_at[row_idx][col_idx] = Some(piece);
                    col_idx += 1;
                }
            }

            if col_idx != 8 {
                return Err(InvalidBoardSetup(fen_pieces.to_string()));
            }
        }

        if row_idx != 0 {
            return Err(InvalidBoardSetup(fen_pieces.to_string()));
        }

        // Turn
        let turn_fen = fen_parts.next().ok_or(MissingField(fen.to_string()))?;
        let turn = match turn_fen {
            "w" => PieceColour::White,
            "b" => PieceColour::Black,
            s => return Err(InvalidTurn(s.to_string())),
        };

        // Castling rights
        let castling_rights_fen = fen_parts.next().ok_or(MissingField(fen.to_string()))?;
        let mut castling_rights = BitBoard::EMPTY;
        for castle in castling_rights_fen.chars() {
            match castle {
                'Q' => castling_rights.set(Square::from_coords(0, 0)),
                'K' => castling_rights.set(Square::from_coords(0, 7)),
                'q' => castling_rights.set(Square::from_coords(7, 0)),
                'k' => castling_rights.set(Square::from_coords(7, 7)),
                '-' => {}
                c => return Err(InvalidCastlingRights(c)),
            }
        }

        let en_passant_fen = fen_parts.next().ok_or(MissingField(fen.to_string()))?;
        let mut en_passant = BitBoard::EMPTY;
        if en_passant_fen != "-" {
            let square = match Square::from_str(en_passant_fen) {
                Ok(sq) => sq,
                Err(e) => return Err(FenError::NotationError(e)),
            };
            en_passant.set(square)
        }

        let halfmove_clock_fen = fen_parts.next().ok_or(MissingField(fen.to_string()))?;
        let halfmove_clock = halfmove_clock_fen.parse().map_err(InvalidHalfmoveClock)?;

        let fullmove_number_fen = fen_parts.next().ok_or(MissingField(fen.to_string()))?;
        let fullmove_number = fullmove_number_fen.parse().map_err(InvalidFullmoveNumber)?;

        Ok(Board {
            bitboards,
            piece_at,
            turn,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Places a piece on both the BitBoard and the `piece_at` array,
    /// and ensures they stay in sync
    fn set_piece(&mut self, piece: Piece, square: Square) {
        self.bitboards[piece.colour as usize][piece.kind as usize].set(square);
        self.piece_at[square.row()][square.col()] = Some(piece);
    }

    /// Removes a piece from both the BitBoard and the `piece_set` array,
    /// and ensures they stay in sync
    /// Piece must already be set
    fn unset_piece(&mut self, square: Square) -> Piece {
        let piece = self.piece_at[square.row()][square.col()]
            .expect("trying to unset a piece from an empty square");
        self.bitboards[piece.colour as usize][piece.kind as usize].unset(square);
        self.piece_at[square.row()][square.col()] = None;
        piece
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.piece_at[square.row()][square.col()]
    }

    pub fn get_pieces(&self) -> Vec<Piece> {
        let mut pieces = Vec::with_capacity(32);
        for piece in self.piece_at.iter().flatten() {
            if let Some(piece) = piece {
                pieces.push(*piece);
            }
        }
        pieces
    }

    pub fn get_pieces_by_colour(&self, colour: PieceColour) -> Vec<Piece> {
        let mut pieces = Vec::with_capacity(32);
        for piece in self.piece_at.iter().flatten() {
            if let Some(piece) = piece {
                pieces.push(*piece);
            }
        }
        pieces
    }

    pub fn apply_move(&mut self, m: Move) {
        debug_assert!(
            self.piece_at[m.from.row()][m.from.col()].is_some(),
            "Moving piece could not be located on board on its square"
        );

        if m.kind == KingCastle {
            let row = m.from.row();
            let king = self.unset_piece(Square::from_coords(row, 4));
            let rook = self.unset_piece(Square::from_coords(row, 7));
            self.set_piece(king, Square::from_coords(row, 6));
            self.set_piece(rook, Square::from_coords(row, 5));
            self.castling_rights.unset(Square::from_coords(row, 0));
            self.castling_rights.unset(Square::from_coords(row, 7));
        } else if m.kind == QueenCastle {
            let row = m.from.row();
            let king = self.unset_piece(Square::from_coords(row, 4));
            let rook = self.unset_piece(Square::from_coords(row, 0));
            self.set_piece(king, Square::from_coords(row, 2));
            self.set_piece(rook, Square::from_coords(row, 3));
            self.castling_rights.unset(Square::from_coords(row, 0));
            self.castling_rights.unset(Square::from_coords(row, 7));
        } else {
            // Remove captured piece if applicable
            if m.is_capture() {
                self.unset_piece(m.to);
            }

            let moving_piece = self.unset_piece(m.from);
            self.set_piece(moving_piece, m.to);
        }

        self.turn = self.turn.other();
        todo!("en passant");
        todo!("halfmove clock");
        todo!("fullmove increment");
    }

    pub fn en_passant_square(&self) -> Option<Square> {
        match self.en_passant {
            BitBoard::EMPTY => None,
            bb => Some(bb.get_square()),
        }
    }

    pub fn get_colour_bitboard(&self, colour: PieceColour) -> BitBoard {
        combine_bitboards(self.bitboards[colour as usize])
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

fn combine_bitboards(bitboards: [BitBoard; 6]) -> BitBoard {
    let mut res = BitBoard::EMPTY;
    for bb in bitboards {
        res |= bb;
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    // New board returns 32 pieces
    #[test]
    fn count_pieces_new_board() {
        let board = Board::start_position();
        assert_eq!(board.get_pieces().len(), 32);
    }
}

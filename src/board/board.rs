use crate::board::bitboard::BitBoard;
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
                        .set(Square::new(row_idx, col_idx));
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
                'Q' => castling_rights.set(Square::new(0, 0)),
                'K' => castling_rights.set(Square::new(0, 7)),
                'q' => castling_rights.set(Square::new(7, 0)),
                'k' => castling_rights.set(Square::new(7, 7)),
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
}

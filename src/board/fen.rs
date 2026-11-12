use crate::board::{BitBoard, Board, Square, combine_bitboards};
use crate::chess::piece::{Piece, PieceColour, PieceKind};
use crate::chess::zobrist::calculate_hash;
use crate::error::FenError;
use crate::error::FenError::{
    InvalidBoardSetup, InvalidCastlingRights, InvalidFullmoveNumber, InvalidHalfmoveClock,
    InvalidTurn, MissingField,
};
use std::str::FromStr;

impl Board {
    pub fn start_position() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("invalid fen")
    }

    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let mut bitboards = [[BitBoard::EMPTY; PieceKind::COUNT]; PieceColour::COUNT];

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
        let mut castling_rights = 0u8;
        for castle in castling_rights_fen.chars() {
            match castle {
                'Q' => castling_rights += 1 << 3,
                'K' => castling_rights += 1 << 2,
                'q' => castling_rights += 1 << 1,
                'k' => castling_rights += 1,
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

        let colour_bitboards = [
            combine_bitboards(bitboards[0]),
            combine_bitboards(bitboards[1]),
        ];

        let hash = calculate_hash(bitboards, turn, castling_rights, en_passant);

        Ok(Board {
            bitboards,
            turn,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
            colour_bitboards,
            hash,
        })
    }
}

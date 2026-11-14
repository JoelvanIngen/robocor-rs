use crate::board::{BitBoard, Board, Square};
use crate::chess::moves::MoveKind::{EnPassant, KingCastle, QueenCastle};
use crate::chess::moves::{Move, MoveKind};
use crate::chess::piece::PieceKind::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::chess::piece::{Piece, PieceColour};
use crate::move_gen::{
    get_attacks_bishop, get_attacks_king, get_attacks_knight, get_attacks_pawn, get_attacks_rook,
};

const DIRECTIONS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];
const IS_DIAGONAL: [bool; 8] = [true, false, true, false, false, true, false, true];

pub struct CheckContext {
    // Enemy pieces giving a check on the king
    pub checkers: BitBoard,
    // Squares that a non-king move must cover to break a check
    // The checker's square and every square between in and the king
    // Is empty when not in check
    // Is empty when in double check as a king move is forced
    pub check_mask: BitBoard,
    // Friendly pieces that are pinned against friendly king
    pub pinned: BitBoard,
    // For each square, if any friendly piece is pinned and on that square,
    // the ray (excluding king square but including its own square) it is
    // allowed to move along
    // Is empty for any non-pinned squares
    pub pinned_rays: [BitBoard; 64],
}

impl Board {
    pub fn check_context(&self, colour: PieceColour) -> CheckContext {
        let enemy = colour.other();
        let friendly_bb = self.get_colour_bitboard(colour);
        let enemy_bb = self.get_colour_bitboard(enemy);
        let king_sq = self.get_bitboard(Piece::new(King, colour)).get_square();

        let enemy_pawns = self.get_bitboard(Piece::new(Pawn, enemy));
        let enemy_knights = self.get_bitboard(Piece::new(Knight, enemy));
        let enemy_bishops = self.get_bitboard(Piece::new(Bishop, enemy));
        let enemy_rooks = self.get_bitboard(Piece::new(Rook, enemy));
        let enemy_queens = self.get_bitboard(Piece::new(Queen, enemy));
        let enemy_king = self.get_bitboard(Piece::new(King, enemy));

        let enemy_orthogonals = enemy_rooks | enemy_queens;
        let enemy_diagonals = enemy_bishops | enemy_queens;

        // Checkers
        let mut checkers = BitBoard::EMPTY;
        checkers |= get_attacks_pawn(king_sq, colour) & enemy_pawns;
        checkers |= get_attacks_knight(king_sq, BitBoard::EMPTY) & enemy_knights;
        checkers |= get_attacks_king(king_sq, BitBoard::EMPTY) & enemy_king; // Might be useless cos king can't give check anyway?

        let rook_from_king = get_attacks_rook(king_sq, friendly_bb, enemy_bb);
        let bishop_from_king = get_attacks_bishop(king_sq, friendly_bb, enemy_bb);
        checkers |= (rook_from_king & enemy_orthogonals) | (bishop_from_king & enemy_diagonals);

        // Check mask
        let check_mask = if checkers.count() == 1 {
            let checker_sq = checkers.get_square();
            squares_between(king_sq, checker_sq) | checkers
        } else {
            BitBoard::EMPTY
        };

        // Pins
        let mut pinned = BitBoard::EMPTY;
        let mut pin_rays = [BitBoard::EMPTY; 64];

        for dir in 0..8 {
            let (dr, dc) = DIRECTIONS[dir];
            let is_diagonal = IS_DIAGONAL[dir];

            let mut row = king_sq.row() as i8 + dr;
            let mut col = king_sq.col() as i8 + dc;
            let mut candidate = None;

            while (0..8).contains(&row) && (0..8).contains(&col) {
                let square = Square::from_coords(row as usize, col as usize);

                if friendly_bb.is_set(square) {
                    if candidate.is_some() {
                        // Not pinned
                        break;
                    }
                    // Track potentially pinned piece
                    candidate = Some(square);
                } else if enemy_bb.is_set(square) {
                    if let Some(pin_candidate) = candidate {
                        let is_pinner = if is_diagonal {
                            enemy_diagonals.is_set(square)
                        } else {
                            enemy_orthogonals.is_set(square)
                        };

                        if is_pinner {
                            pinned.set(pin_candidate);
                            pin_rays[pin_candidate.idx()] =
                                ray_between_inclusive(king_sq, square, dr, dc);
                        }
                    }

                    break;
                }
                row += dr;
                col += dc;
            }
        }

        CheckContext {
            checkers,
            check_mask,
            pinned,
            pinned_rays: pin_rays,
        }
    }

    pub fn is_legal(&self, m: Move, colour: PieceColour, ctx: &CheckContext) -> bool {
        let kind = m.kind();
        let from = m.from();
        let to = m.to();

        if kind == KingCastle || kind == QueenCastle {
            return self.is_legal_castle(colour, kind, ctx);
        }
        if kind == EnPassant {
            return self.is_legal_en_passant(m, colour);
        }

        let king_bb = self.get_bitboard(Piece::new(King, colour));

        if king_bb.is_set(from) {
            self.is_safe_king_square(from, to, colour)
        } else if ctx.checkers.count() > 1 {
            // King move is forced instead
            false
        } else if ctx.pinned.is_set(from) {
            // Pinned pieces can only move along their pin ray and they must resolve a check if in check
            ctx.pinned_rays[from.idx()].is_set(to)
                && (ctx.checkers == BitBoard::EMPTY || ctx.check_mask.is_set(to))
        } else {
            // If in check, move must block or capture checker
            ctx.checkers == BitBoard::EMPTY || ctx.check_mask.is_set(to)
        }
    }

    /// Direct check test on the king's new square
    fn is_safe_king_square(&self, from: Square, to: Square, colour: PieceColour) -> bool {
        let enemy = colour.other();

        // Move king
        let mut friendly_after = self.get_colour_bitboard(colour);
        friendly_after.unset(from);
        friendly_after.set(to);
        let enemy_bb = self.get_colour_bitboard(enemy);

        // Sliding attacks
        let rook_attacks = get_attacks_rook(to, friendly_after, enemy_bb);
        let bishop_attacks = get_attacks_bishop(to, friendly_after, enemy_bb);

        let enemy_orthogonal = self.get_bitboard(Piece::new(Rook, enemy))
            | self.get_bitboard(Piece::new(Queen, enemy));
        let enemy_diagonal = self.get_bitboard(Piece::new(Bishop, enemy))
            | self.get_bitboard(Piece::new(Queen, enemy));

        if (rook_attacks & enemy_orthogonal) != BitBoard::EMPTY {
            return false;
        }
        if (bishop_attacks & enemy_diagonal) != BitBoard::EMPTY {
            return false;
        }

        // Non-sliding attacks
        if (get_attacks_pawn(to, colour) & self.get_bitboard(Piece::new(Pawn, enemy)))
            != BitBoard::EMPTY
        {
            return false;
        }
        if (get_attacks_knight(to, BitBoard::EMPTY) & self.get_bitboard(Piece::new(Knight, enemy)))
            != BitBoard::EMPTY
        {
            return false;
        }
        if (get_attacks_king(to, BitBoard::EMPTY) & self.get_bitboard(Piece::new(King, enemy)))
            != BitBoard::EMPTY
        {
            return false;
        }

        true
    }

    /// Test if en passant leaves own king in check
    fn is_legal_en_passant(&self, m: Move, colour: PieceColour) -> bool {
        let enemy = colour.other();
        let king_sq = self.get_bitboard(Piece::new(King, colour)).get_square();
        let capture_sq = Square::from_coords(m.from().row(), m.to().col());

        // Remove pawn and move ours
        let mut friendly_after = self.get_colour_bitboard(colour);
        friendly_after.unset(m.from());
        friendly_after.set(m.to());
        let mut enemy_after = self.get_colour_bitboard(enemy);
        enemy_after.unset(capture_sq);

        // Sliding attacks
        let rook_attacks = get_attacks_rook(king_sq, friendly_after, enemy_after);
        let bishop_attacks = get_attacks_bishop(king_sq, friendly_after, enemy_after);

        let captured_bb = BitBoard::from_u64(1u64 << capture_sq.idx());
        let enemy_orthogonal = (self.get_bitboard(Piece::new(Rook, enemy))
            | self.get_bitboard(Piece::new(Queen, enemy)))
            & !captured_bb;
        let enemy_diagonal = (self.get_bitboard(Piece::new(Bishop, enemy))
            | self.get_bitboard(Piece::new(Queen, enemy)))
            & !captured_bb;

        if (rook_attacks & enemy_orthogonal) != BitBoard::EMPTY {
            return false;
        }
        if (bishop_attacks & enemy_diagonal) != BitBoard::EMPTY {
            return false;
        }

        // Non-sliding attacks
        let enemy_pawns_after = self.get_bitboard(Piece::new(Pawn, enemy)) & !captured_bb;
        if (get_attacks_pawn(king_sq, colour) & enemy_pawns_after) != BitBoard::EMPTY {
            return false;
        }
        if (get_attacks_knight(king_sq, BitBoard::EMPTY)
            & self.get_bitboard(Piece::new(Knight, enemy)))
            != BitBoard::EMPTY
        {
            return false;
        }
        if (get_attacks_king(king_sq, BitBoard::EMPTY) & self.get_bitboard(Piece::new(King, enemy)))
            != BitBoard::EMPTY
        {
            return false;
        }

        true
    }

    fn is_legal_castle(&self, colour: PieceColour, kind: MoveKind, ctx: &CheckContext) -> bool {
        // Castling out of check is illegal
        if ctx.checkers != BitBoard::EMPTY {
            return false;
        }

        // Destination square or pass-through square cannot be attacked either
        let row = if colour == PieceColour::White { 0 } else { 7 };
        let path = match kind {
            KingCastle => [Square::from_coords(row, 5), Square::from_coords(row, 6)],
            QueenCastle => [Square::from_coords(row, 3), Square::from_coords(row, 2)],
            _ => unreachable!(),
        };

        for sq in path {
            if self.is_bb_attacked(colour, BitBoard::from_u64(1u64 << sq.idx())) {
                return false;
            }
        }

        true
    }
}

/// Squares between square a and square b, exclusive on both ends
/// Empty is not aligned
fn squares_between(a: Square, b: Square) -> BitBoard {
    let (ar, ac) = (a.row() as i32, a.col() as i32);
    let (br, bc) = (b.row() as i32, b.col() as i32);
    let (dr, dc) = ((br - ar).signum(), (bc - ac).signum());
    let (rd, cd) = ((br - ar).abs(), (bc - ac).abs());

    debug_assert!(
        !(rd == 0 && cd == 0),
        "Squares between fired on two equal squares"
    );

    if !(rd == 0 || cd == 0 || rd == cd) {
        // Neither orthogonal nor vertical
        return BitBoard::EMPTY;
    }

    let mut bb = BitBoard::EMPTY;
    let (mut r, mut c) = (ar + dr, ac + dc);
    while (r, c) != (br, bc) {
        bb.set(Square::from_coords(r as usize, c as usize));
        r += dr;
        c += dc;
    }
    bb
}

/// Squares between king (exclusive) and pinner (inclusive)
fn ray_between_inclusive(king: Square, pinner: Square, dr: i8, dc: i8) -> BitBoard {
    let mut bb = BitBoard::EMPTY;
    let mut r = king.row() as i8 + dr;
    let mut c = king.col() as i8 + dc;
    while (r as usize, c as usize) != (pinner.row(), pinner.col()) {
        bb.set(Square::from_coords(r as usize, c as usize));
        r += dr;
        c += dc;
    }
    bb.set(pinner);
    bb
}

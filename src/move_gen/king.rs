use crate::board::Square;
use crate::chess::bitboard::BitBoard;
use crate::chess::moves::MoveKind::{Capture, KingCastle, QueenCastle, QuietMove};
use crate::chess::moves::{Move, MoveList};
use arrayvec::ArrayVec;

const KING_DELTAS: [(i8, i8); 8] = [
    // (ROW, COL)
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// Squares that need to be unoccupied for castling
const QUEENSIDE_CASTLE_SQUARES: [BitBoard; 2] = [
    {
        let mut bb = BitBoard::EMPTY;
        bb.set(Square::from_coords(0, 1));
        bb.set(Square::from_coords(0, 2));
        bb.set(Square::from_coords(0, 3));
        bb
    },
    {
        let mut bb = BitBoard::EMPTY;
        bb.set(Square::from_coords(7, 1));
        bb.set(Square::from_coords(7, 2));
        bb.set(Square::from_coords(7, 3));
        bb
    },
];
const KINGSIDE_CASTLE_SQUARES: [BitBoard; 2] = [
    {
        let mut bb = BitBoard::EMPTY;
        bb.set(Square::from_coords(0, 5));
        bb.set(Square::from_coords(0, 6));
        bb
    },
    {
        let mut bb = BitBoard::EMPTY;
        bb.set(Square::from_coords(7, 5));
        bb.set(Square::from_coords(7, 6));
        bb
    },
];

pub const KING_ATTACKS: [BitBoard; 64] = {
    let mut attacks = [BitBoard::EMPTY; 64];

    let mut i = 0;
    while i < 64 {
        let pos = Square::from_index(i);
        let mut bb = BitBoard::EMPTY;

        let mut d = 0;
        while d < KING_DELTAS.len() {
            let (dr, dc) = KING_DELTAS[d];
            let r = pos.row() as i8 + dr;
            let c = pos.col() as i8 + dc;

            if r >= 0 && r < 8 && c >= 0 && c < 8 {
                let target = Square::from_coords(r as usize, c as usize);
                bb.set(target);
            }

            d += 1;
        }

        attacks[i] = bb;
        i += 1;
    }

    attacks
};

#[inline(always)]
pub fn get_attacks_king(pos: Square, friendly: BitBoard) -> BitBoard {
    // `& !friendly` might not matter as the presence of a friendly piece
    // means no enemy piece is present
    // I'll keep it for the time being but removing might save two operations
    KING_ATTACKS[pos.idx()] & !friendly
}

pub fn get_pl_moves_king(
    moves: &mut MoveList,
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    castling_options: ArrayVec<Square, 2>,
) {
    let targets = get_attacks_king(pos, friendly);
    let captures = targets & enemy;
    let quiets = targets & !enemy;

    for to in captures.iter() {
        moves.push(Move::new(Capture, pos, to));
    }

    for to in quiets.iter() {
        moves.push(Move::new(QuietMove, pos, to));
    }

    get_pl_castling_moves(moves, pos, friendly, enemy, castling_options);
}

fn get_pl_castling_moves(
    moves: &mut MoveList,
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    castling_options: ArrayVec<Square, 2>,
) {
    let full_bb = friendly | enemy;
    for co in castling_options {
        // This is very hacky, but white (row 0) will be 0
        // and black (row 7) will be 1
        // which corresponds to our data format
        let colour_idx = co.row() % 2;

        // Make sure squares between king and rook are empty, we'll check later whether the squares are attacked
        if co.col() == 0 && QUEENSIDE_CASTLE_SQUARES[colour_idx] & full_bb == BitBoard::EMPTY {
            // Queenside
            moves.push(Move::new(
                QueenCastle,
                pos,
                Square::from_coords(co.row(), 2),
            ));
        }
        if co.col() == 7 && KINGSIDE_CASTLE_SQUARES[colour_idx] & full_bb == BitBoard::EMPTY {
            // Kingside
            moves.push(Move::new(KingCastle, pos, Square::from_coords(co.row(), 6)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a1_empty() {
        let orig_square = Square::from_coords(0, 0);
        let want = vec![
            Move::new(QuietMove, orig_square, Square::from_coords(0, 1)),
            Move::new(QuietMove, orig_square, Square::from_coords(1, 0)),
            Move::new(QuietMove, orig_square, Square::from_coords(1, 1)),
        ];
        let mut got = MoveList::new();
        get_pl_moves_king(
            &mut got,
            orig_square,
            BitBoard::EMPTY,
            BitBoard::EMPTY,
            ArrayVec::<Square, 2>::new(),
        );

        assert_eq!(
            want.len(),
            got.len(),
            "move count mismatch: {got:?} vs {want:?}"
        );
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }

    #[test]
    fn test_a1_friendly_targets() {
        let orig_square = Square::from_coords(0, 0);
        let mut friendly = BitBoard::EMPTY;
        friendly.set(Square::from_coords(0, 1));
        friendly.set(Square::from_coords(1, 0));
        friendly.set(Square::from_coords(1, 1));

        let mut moves = MoveList::new();
        get_pl_moves_king(
            &mut moves,
            orig_square,
            friendly,
            BitBoard::EMPTY,
            ArrayVec::<Square, 2>::new(),
        );
        assert_eq!(moves.len(), 0, "is moving through friendly pieces");
    }

    #[test]
    fn test_a1_enemy_targets() {
        let orig_square = Square::from_coords(0, 0);
        let friendly = BitBoard::EMPTY;
        let mut enemy = BitBoard::EMPTY;
        enemy.set(Square::from_coords(0, 1));
        enemy.set(Square::from_coords(1, 0));
        enemy.set(Square::from_coords(1, 1));

        let want = ArrayVec::from([
            Move::new(Capture, orig_square, Square::from_coords(0, 1)),
            Move::new(Capture, orig_square, Square::from_coords(1, 0)),
            Move::new(Capture, orig_square, Square::from_coords(1, 1)),
        ]);
        let mut got = MoveList::new();
        get_pl_moves_king(
            &mut got,
            orig_square,
            friendly,
            enemy,
            ArrayVec::<Square, 2>::new(),
        );

        assert_eq!(want.len(), got.len());
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }

    // Ability to castle when nothing is in the way
    #[test]
    fn test_castling_flags_not_blocked() {
        let orig_square = Square::from_coords(0, 4);
        let friendly = BitBoard::EMPTY;
        let enemy = BitBoard::EMPTY;

        let want = vec![
            Move::new(QueenCastle, orig_square, Square::from_coords(0, 2)),
            Move::new(KingCastle, orig_square, Square::from_coords(0, 6)),
        ];
        let mut got = MoveList::new();
        get_pl_castling_moves(
            &mut got,
            orig_square,
            friendly,
            enemy,
            ArrayVec::from([Square::from_coords(0, 0), Square::from_coords(0, 7)]),
        );

        assert_eq!(want.len(), got.len());
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }

    // Ability to castle when some friendly piece blocks the move
    #[test]
    fn test_castling_flags_blocked() {
        let orig_square = Square::from_coords(0, 4);
        let mut friendly = BitBoard::EMPTY;
        let enemy = BitBoard::EMPTY;

        friendly.set(Square::from_coords(0, 1));
        friendly.set(Square::from_coords(0, 5));

        let want = vec![];
        let mut got = MoveList::new();
        get_pl_castling_moves(
            &mut got,
            orig_square,
            friendly,
            enemy,
            ArrayVec::from([Square::from_coords(0, 0), Square::from_coords(0, 7)]),
        );

        assert_eq!(want.len(), got.len());
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }
}

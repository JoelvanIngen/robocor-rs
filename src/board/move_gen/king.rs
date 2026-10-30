use crate::board::Square;
use crate::board::bitboard::BitBoard;
use crate::board::moves::Move;
use crate::board::moves::MoveKind::{Capture, KingCastle, QueenCastle, QuietMove};

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

pub fn get_pl_moves_king(
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    castling_options: Vec<Square>,
) -> Vec<Move> {
    let mut moves = KING_DELTAS
        .iter()
        // Generate all new candidate positions
        .map(|delta| [pos.row() as i8 + delta.0, pos.col() as i8 + delta.1])
        // Ensure candidates are within board
        .filter(|p| (0..8).contains(&p[0]) && (0..8).contains(&p[1]))
        // Convert to square types
        .map(|p| Square::from_coords(p[0] as usize, p[1] as usize))
        // Drop squares occupied by friendly piece
        .filter(|s| !friendly.is_set(*s))
        // Convert to move; capture if occupied by enemy
        .map(|new_pos| {
            Move::new(
                match enemy.is_set(new_pos) {
                    true => Capture,
                    false => QuietMove,
                },
                pos,
                new_pos,
            )
        })
        .collect();

    get_pl_castling_moves(&mut moves, pos, friendly, enemy, castling_options);

    moves
}

fn get_pl_castling_moves(
    moves: &mut Vec<Move>,
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    castling_options: Vec<Square>,
) {
    let full_bb = friendly | enemy;
    for co in castling_options {
        // Make sure squares between king and rook are empty, we'll check later whether the squares are attacked
        if co.col() == 0
            && !full_bb.is_set(Square::from_coords(co.row(), 1))
            && !full_bb.is_set(Square::from_coords(co.row(), 2))
            && !full_bb.is_set(Square::from_coords(co.row(), 3))
        {
            // Queenside
            moves.push(Move::new(
                QueenCastle,
                pos,
                Square::from_coords(co.row(), 0),
            ));
        }
        if co.col() == 7
            && !full_bb.is_set(Square::from_coords(co.row(), 6))
            && !full_bb.is_set(Square::from_coords(co.row(), 5))
        {
            // Kingside
            moves.push(Move::new(KingCastle, pos, Square::from_coords(co.row(), 7)));
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
        let got = get_pl_moves_king(orig_square, BitBoard::EMPTY, BitBoard::EMPTY, vec![]);

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

        let moves = get_pl_moves_king(orig_square, friendly, BitBoard::EMPTY, vec![]);
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

        let want = vec![
            Move::new(Capture, orig_square, Square::from_coords(0, 1)),
            Move::new(Capture, orig_square, Square::from_coords(1, 0)),
            Move::new(Capture, orig_square, Square::from_coords(1, 1)),
        ];
        let got = get_pl_moves_king(orig_square, friendly, enemy, vec![]);

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
            Move::new(QueenCastle, orig_square, Square::from_coords(0, 0)),
            Move::new(KingCastle, orig_square, Square::from_coords(0, 7)),
        ];
        let mut got = vec![];
        get_pl_castling_moves(
            &mut got,
            orig_square,
            friendly,
            enemy,
            vec![Square::from_coords(0, 0), Square::from_coords(0, 7)],
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
        let mut got = vec![];
        get_pl_castling_moves(
            &mut got,
            orig_square,
            friendly,
            enemy,
            vec![Square::from_coords(0, 0), Square::from_coords(0, 7)],
        );

        assert_eq!(want.len(), got.len());
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }
}

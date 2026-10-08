use crate::board::Square;
use crate::board::bitboard::BitBoard;
use crate::board::moves::Move;
use crate::board::moves::MoveKind::{Capture, QuietMove};

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

pub fn get_pl_moves_king(pos: Square, friendly: BitBoard, enemy: BitBoard) -> Vec<Move> {
    let mut moves = Vec::with_capacity(KING_DELTAS.len());

    for delta in KING_DELTAS {
        let new_row = pos.row() as i8 + delta.0;
        if !(0..8).contains(&new_row) {
            continue;
        }
        let new_col = pos.col() as i8 + delta.1;
        if !(0..8).contains(&new_col) {
            continue;
        }
        let new_pos = Square::from_coords(new_row as usize, new_col as usize);
        if friendly.is_set(new_pos) {
            continue;
        }

        let move_kind = match enemy.is_set(new_pos) {
            true => Capture,
            false => QuietMove,
        };

        moves.push(Move::new(move_kind, pos, new_pos));
    }

    moves
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
        let got = get_pl_moves_king(orig_square, BitBoard::EMPTY, BitBoard::EMPTY);

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

        let moves = get_pl_moves_king(orig_square, friendly, BitBoard::EMPTY);
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
        let got = get_pl_moves_king(orig_square, friendly, enemy);

        assert_eq!(want.len(), got.len());
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }
}

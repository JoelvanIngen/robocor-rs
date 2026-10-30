use crate::board::move_gen::common::get_pl_moves_direction;
use crate::board::moves::Move;
use crate::board::{BitBoard, Square};

const DIRECTIONS: [(i8, i8); 4] = [
    // (row delta, col delta)
    (-1, 0),
    (0, -1),
    (0, 1),
    (1, 0),
];

pub fn get_attacks_rook(pos: Square, friendly: BitBoard, enemy: BitBoard) -> BitBoard {
    // Wrong way around (ideally we compute bitboard cheap instead of computing entire moves)
    // but that's optimising for later
    DIRECTIONS
        .iter()
        .flat_map(|dir| get_pl_moves_direction(pos, friendly, enemy, *dir))
        .fold(BitBoard::EMPTY, |mut bb, m| {
            bb.set(m.to);
            bb
        })
}

pub fn get_pl_moves_rook(pos: Square, friendly: BitBoard, enemy: BitBoard) -> Vec<Move> {
    let mut moves = vec![];
    for dir in DIRECTIONS {
        moves.extend(get_pl_moves_direction(pos, friendly, enemy, dir));
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::moves::MoveKind::{Capture, QuietMove};
    use std::str::FromStr;

    struct TestCase {
        pos: &'static str,
        friendly: &'static [&'static str],
        enemy: &'static [&'static str],
        expected_moves: Vec<Move>,
    }

    fn run_test(case: TestCase) {
        let pos = Square::from_str(case.pos).unwrap();
        let mut friendly = BitBoard::EMPTY;
        for &sq in case.friendly {
            friendly.set(Square::from_str(sq).unwrap());
        }
        let mut enemy = BitBoard::EMPTY;
        for &sq in case.enemy {
            enemy.set(Square::from_str(sq).unwrap());
        }

        let got = get_pl_moves_rook(pos, friendly, enemy);
        let want = case.expected_moves;

        assert_eq!(
            got.len(),
            want.len(),
            "move count mismatch: {got:?} vs {want:?}"
        );
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }

    // Test rook in middle of board
    #[test]
    fn test_middle() {
        let orig_square_str = "e5";
        let orig_square = Square::from_str(orig_square_str).unwrap();
        run_test(TestCase {
            pos: orig_square_str,
            friendly: &[],
            enemy: &[],
            expected_moves: vec![
                Move::new(QuietMove, orig_square, Square::from_str("e6").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("e7").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("e8").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("e4").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("e3").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("e2").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("e1").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("d5").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("c5").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("b5").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("a5").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("f5").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("g5").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("h5").unwrap()),
            ],
        })
    }

    // Rook a1 with friendly blocked b1 and a2 (no legal moves)
    #[test]
    fn test_a1_blocked_friendly() {
        run_test(TestCase {
            pos: "a1",
            friendly: &["a2", "b1"],
            enemy: &[],
            expected_moves: vec![],
        })
    }

    // Rook a1 with enemy blocked b1 and a2 (b1 and a2 only legal move)
    #[test]
    fn test_a1_blocked_enemy() {
        let orig_square_str = "a1";
        let orig_square = Square::from_str(orig_square_str).unwrap();
        run_test(TestCase {
            pos: orig_square_str,
            friendly: &[],
            enemy: &["a2", "b1"],
            expected_moves: vec![
                Move::new(Capture, orig_square, Square::from_str("a2").unwrap()),
                Move::new(Capture, orig_square, Square::from_str("b1").unwrap()),
            ],
        })
    }
}

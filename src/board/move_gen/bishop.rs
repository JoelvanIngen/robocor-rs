use crate::board::moves::Move;
use crate::board::moves::MoveKind::{Capture, QuietMove};
use crate::board::{BitBoard, Square};

const DIRECTIONS: [(i8, i8); 4] = [
    // (row delta, col delta)
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];

pub fn get_pl_moves_bishop(pos: Square, friendly: BitBoard, enemy: BitBoard) -> Vec<Move> {
    let mut moves = vec![];
    for dir in DIRECTIONS {
        get_pl_moves_direction(pos, friendly, enemy, dir, &mut moves);
    }
    moves
}

/// Iterate in a given direction until stopped or reaching the end of the board
/// Returns all pseudo legal moves in that direction
fn get_pl_moves_direction(
    orig_pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    dir: (i8, i8),
    moves: &mut Vec<Move>,
) {
    let mut pos = orig_pos;
    loop {
        let new_row = pos.row() as i8 + dir.0;
        let new_col = pos.col() as i8 + dir.1;
        pos = match is_within_board(new_row, new_col) {
            true => Square::from_coords(new_row as usize, new_col as usize),
            false => return,
        };

        if friendly.is_set(pos) {
            return;
        }

        if enemy.is_set(pos) {
            moves.push(Move::new(Capture, orig_pos, pos));
            return;
        }

        moves.push(Move::new(QuietMove, orig_pos, pos));
    }
}

fn is_within_board(row: i8, col: i8) -> bool {
    (0..8).contains(&row) && (0..8).contains(&col)
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let got = get_pl_moves_bishop(pos, friendly, enemy);
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

    // Test bishop in middle of board
    #[test]
    fn test_middle() {
        let orig_square_str = "e5";
        let orig_square = Square::from_str(orig_square_str).unwrap();
        run_test(TestCase {
            pos: orig_square_str,
            friendly: &[],
            enemy: &[],
            expected_moves: vec![
                Move::new(QuietMove, orig_square, Square::from_str("d4").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("c3").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("b2").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("a1").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("f6").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("g7").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("h8").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("d6").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("c7").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("b8").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("f4").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("g3").unwrap()),
                Move::new(QuietMove, orig_square, Square::from_str("h2").unwrap()),
            ],
        })
    }

    // Bishop a1 with friendly blocked b2 (no legal moves)
    #[test]
    fn test_a1_blocked_friendly() {
        run_test(TestCase {
            pos: "a1",
            friendly: &["b2"],
            enemy: &[],
            expected_moves: vec![],
        })
    }

    // Bishop a1 with enemy blocked b2 (b2 only legal move)
    #[test]
    fn test_a1_blocked_enemy() {
        run_test(TestCase {
            pos: "a1",
            friendly: &[],
            enemy: &["b2"],
            expected_moves: vec![Move::new(
                Capture,
                Square::from_str("a1").unwrap(),
                Square::from_str("b2").unwrap(),
            )],
        })
    }
}

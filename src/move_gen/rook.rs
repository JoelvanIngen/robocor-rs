use crate::board::{BitBoard, Square};
use crate::chess::moves::MoveKind::{Capture, QuietMove};
use crate::chess::moves::{Move, MoveList};
use crate::move_gen::magic::{ROOK_ATTACKS, ROOK_MAGICS, ROOK_MASKS, SHIFT};

const DIRECTIONS: [(i8, i8); 4] = [
    // (row delta, col delta)
    (-1, 0),
    (0, -1),
    (0, 1),
    (1, 0),
];

pub const fn slow_rook_attacks(square_idx: usize, occ: u64) -> u64 {
    let row = square_idx / 8;
    let col = square_idx % 8;
    let mut attacks = 0u64;

    // Up
    let mut new_row = row + 1;
    while new_row < 8 {
        let bit = 1u64 << (new_row * 8 + col);
        attacks |= bit;
        if occ & bit != 0 {
            break;
        }
        new_row += 1;
    }

    // Down
    let mut new_row = row as i32 - 1;
    while new_row >= 0 {
        let bit = 1u64 << (new_row as usize * 8 + col);
        attacks |= bit;
        if occ & bit != 0 {
            break;
        }
        new_row -= 1;
    }

    // Right
    let mut new_col = col + 1;
    while new_col < 8 {
        let bit = 1u64 << (row * 8 + new_col);
        attacks |= bit;
        if occ & bit != 0 {
            break;
        }
        new_col += 1;
    }

    // Left
    let mut new_col = col as i32 - 1;
    while new_col >= 0 {
        let bit = 1u64 << (row * 8 + new_col as usize);
        attacks |= bit;
        if occ & bit != 0 {
            break;
        }
        new_col -= 1;
    }

    attacks
}

pub fn get_attacks_rook(pos: Square, friendly: BitBoard, enemy: BitBoard) -> BitBoard {
    let sq = pos.idx();
    let occ = (friendly | enemy).to_u64() & ROOK_MASKS[sq];
    let idx = ((occ.wrapping_mul(ROOK_MAGICS[sq])) >> SHIFT) as usize;
    ROOK_ATTACKS[sq][idx]
}

pub fn get_pl_moves_rook(moves: &mut MoveList, pos: Square, friendly: BitBoard, enemy: BitBoard) {
    let attacks = get_attacks_rook(pos, friendly, enemy);
    for to in (attacks & enemy).iter() {
        moves.push(Move::new(Capture, pos, to));
    }
    for to in (attacks & !friendly & !enemy).iter() {
        moves.push(Move::new(QuietMove, pos, to));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::moves::Move;
    use crate::chess::moves::MoveKind::{Capture, QuietMove};
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

        let mut got = MoveList::new();
        get_pl_moves_rook(&mut got, pos, friendly, enemy);
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

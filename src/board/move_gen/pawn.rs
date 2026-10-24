use crate::board::moves::MoveKind::{
    BishopPromotion, BishopPromotionCapture, Capture, DoublePawnPush, EnPassant, KnightPromotion,
    KnightPromotionCapture, QueenPromotion, QueenPromotionCapture, QuietMove, RookPromotion,
    RookPromotionCapture,
};
use crate::board::moves::{Move, MoveKind};
use crate::board::piece::PieceColour;
use crate::board::piece::PieceColour::{Black, White};
use crate::board::{BitBoard, Square};

const PROMOTION_VARIANTS: [MoveKind; 4] = [
    KnightPromotion,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
];

const CAPTURE_PROMOTION_VARIANTS: [MoveKind; 4] = [
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture,
];

pub fn get_pl_moves_pawn(
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    colour: PieceColour,
    ep_square: Option<Square>,
) -> Vec<Move> {
    // Pre-allocate space for single/double push and both captures
    let mut moves = Vec::with_capacity(4);

    // Push if not blocked
    let push_row = match colour {
        White => pos.row() + 1,
        Black => pos.row() - 1,
    };
    let push_pos = Square::from_coords(push_row, pos.col());
    if !friendly.is_set(push_pos) && !enemy.is_set(push_pos) {
        // No pieces in the way of normal push
        if (1..7).contains(&push_pos.row()) {
            moves.push(Move::new(QuietMove, pos, push_pos));
        } else {
            // Promotion
            for variant in PROMOTION_VARIANTS {
                moves.push(Move::new(variant, pos, push_pos));
            }
        }

        // Double push iff pawn is on start row and not blocked
        let start_row = match colour {
            White => 1,
            Black => 6,
        };
        if pos.row() == start_row {
            let double_push_row = match colour {
                White => pos.row() + 2,
                Black => pos.row() - 2,
            };
            let double_push_pos = Square::from_coords(double_push_row, pos.col());
            if !friendly.is_set(double_push_pos) && !enemy.is_set(double_push_pos) {
                moves.push(Move::new(DoublePawnPush, pos, double_push_pos))
            }
        }
    }

    for capture_col_delta in [-1i8, 1i8] {
        let new_col = pos.col() as i8 + capture_col_delta;
        debug_assert!((0..=7).contains(&new_col));

        let capture_pos = Square::from_coords(push_row, new_col as usize);

        // En passant
        if let Some(ep) = ep_square
            && ep == capture_pos
        {
            moves.push(Move::new(EnPassant, pos, capture_pos));
            continue; // EP square is always empty and non-promotion
        }

        if !enemy.is_set(capture_pos) {
            // Cannot capture on an empty square
            continue;
        }

        if (1..7).contains(&capture_pos.row()) {
            moves.push(Move::new(Capture, pos, capture_pos));
        } else {
            for variant in CAPTURE_PROMOTION_VARIANTS {
                moves.push(Move::new(variant, pos, capture_pos));
            }
        }
    }

    moves
}

#[cfg(test)]
mod test {
    use crate::board::move_gen::pawn::get_pl_moves_pawn;
    use crate::board::moves::Move;
    use crate::board::moves::MoveKind::{
        BishopPromotion, BishopPromotionCapture, Capture, DoublePawnPush, KnightPromotion,
        KnightPromotionCapture, QueenPromotion, QueenPromotionCapture, QuietMove, RookPromotion,
        RookPromotionCapture,
    };
    use crate::board::piece::PieceColour;
    use crate::board::piece::PieceColour::{Black, White};
    use crate::board::{BitBoard, Square};
    use std::str::FromStr;

    struct PawnTestCase {
        pos: &'static str,
        color: PieceColour,
        friendly: &'static [&'static str],
        enemy: &'static [&'static str],
        ep: Option<&'static str>,
        expected_moves: Vec<Move>,
    }

    fn run_pawn_test(case: PawnTestCase) {
        let pos = Square::from_str(case.pos).unwrap();
        let mut friendly = BitBoard::EMPTY;
        for &sq in case.friendly {
            friendly.set(Square::from_str(sq).unwrap());
        }
        let mut enemy = BitBoard::EMPTY;
        for &sq in case.enemy {
            enemy.set(Square::from_str(sq).unwrap());
        }
        let ep = case.ep.map(|s| Square::from_str(s).unwrap());

        let got = get_pl_moves_pawn(pos, friendly, enemy, case.color, ep);
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

    // Double push + push
    #[test]
    fn test_push_double_push() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: White,
            friendly: &[],
            enemy: &[],
            ep: None,
            expected_moves: vec![
                Move::new(
                    QuietMove,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e3").unwrap(),
                ),
                Move::new(
                    DoublePawnPush,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e4").unwrap(),
                ),
            ],
        })
    }

    // Double push + push + capture on either side
    #[test]
    fn test_push_double_push_capture() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: White,
            friendly: &[],
            enemy: &["d3", "f3"],
            ep: None,
            expected_moves: vec![
                Move::new(
                    QuietMove,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e3").unwrap(),
                ),
                Move::new(
                    DoublePawnPush,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e4").unwrap(),
                ),
                Move::new(
                    Capture,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("d3").unwrap(),
                ),
                Move::new(
                    Capture,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("f3").unwrap(),
                ),
            ],
        })
    }

    // Push but blocked double push by friendly
    #[test]
    fn test_blocked_double_push_friendly() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: White,
            friendly: &["e4"],
            enemy: &[],
            ep: None,
            expected_moves: vec![Move::new(
                QuietMove,
                Square::from_str("e2").unwrap(),
                Square::from_str("e3").unwrap(),
            )],
        })
    }

    // Push but blocked double push by enemy
    #[test]
    fn test_blocked_double_push_enemy() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: White,
            friendly: &[],
            enemy: &["e4"],
            ep: None,
            expected_moves: vec![Move::new(
                QuietMove,
                Square::from_str("e2").unwrap(),
                Square::from_str("e3").unwrap(),
            )],
        })
    }

    // Blocked push (implicitly blocked double push) and no captures
    #[test]
    fn test_blocked_push_enemy() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: White,
            friendly: &[],
            enemy: &["e3"],
            ep: None,
            expected_moves: vec![],
        })
    }

    // Promotion
    #[test]
    fn test_promotion_white() {
        run_pawn_test(PawnTestCase {
            pos: "e7",
            color: White,
            friendly: &[],
            enemy: &[],
            ep: None,
            expected_moves: vec![
                Move::new(
                    QueenPromotion,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("e8").unwrap(),
                ),
                Move::new(
                    RookPromotion,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("e8").unwrap(),
                ),
                Move::new(
                    BishopPromotion,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("e8").unwrap(),
                ),
                Move::new(
                    KnightPromotion,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("e8").unwrap(),
                ),
            ],
        })
    }
    #[test]
    fn test_promotion_black() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: Black,
            friendly: &[],
            enemy: &[],
            ep: None,
            expected_moves: vec![
                Move::new(
                    QueenPromotion,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e1").unwrap(),
                ),
                Move::new(
                    RookPromotion,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e1").unwrap(),
                ),
                Move::new(
                    BishopPromotion,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e1").unwrap(),
                ),
                Move::new(
                    KnightPromotion,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("e1").unwrap(),
                ),
            ],
        })
    }

    // Promotion blocked
    #[test]
    fn test_promotion_blocked_white() {
        run_pawn_test(PawnTestCase {
            pos: "e7",
            color: White,
            friendly: &["e8"],
            enemy: &[],
            ep: None,
            expected_moves: vec![],
        })
    }
    #[test]
    fn test_promotion_blocked_black() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: Black,
            friendly: &["e1"],
            enemy: &[],
            ep: None,
            expected_moves: vec![],
        })
    }

    // Capture promotion
    #[test]
    fn test_capture_promotion_white() {
        run_pawn_test(PawnTestCase {
            pos: "e7",
            color: White,
            friendly: &["e8"],
            enemy: &["d8"],
            ep: None,
            expected_moves: vec![
                Move::new(
                    QueenPromotionCapture,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("d8").unwrap(),
                ),
                Move::new(
                    RookPromotionCapture,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("d8").unwrap(),
                ),
                Move::new(
                    BishopPromotionCapture,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("d8").unwrap(),
                ),
                Move::new(
                    KnightPromotionCapture,
                    Square::from_str("e7").unwrap(),
                    Square::from_str("d8").unwrap(),
                ),
            ],
        })
    }
    #[test]
    fn test_capture_promotion_black() {
        run_pawn_test(PawnTestCase {
            pos: "e2",
            color: Black,
            friendly: &["e1"],
            enemy: &["d1"],
            ep: None,
            expected_moves: vec![
                Move::new(
                    QueenPromotionCapture,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("d1").unwrap(),
                ),
                Move::new(
                    RookPromotionCapture,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("d1").unwrap(),
                ),
                Move::new(
                    BishopPromotionCapture,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("d1").unwrap(),
                ),
                Move::new(
                    KnightPromotionCapture,
                    Square::from_str("e2").unwrap(),
                    Square::from_str("d1").unwrap(),
                ),
            ],
        })
    }
}

use crate::board::{BitBoard, Square};
use crate::chess::moves::MoveKind::{
    BishopPromotion, BishopPromotionCapture, Capture, DoublePawnPush, EnPassant, KnightPromotion,
    KnightPromotionCapture, QueenPromotion, QueenPromotionCapture, QuietMove, RookPromotion,
    RookPromotionCapture,
};
use crate::chess::moves::{Move, MoveKind, MoveList};
use crate::chess::piece::PieceColour;
use crate::chess::piece::PieceColour::{Black, White};

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

/// Pawn attacks [attacker colour] [starting square idx]
const PAWN_ATTACKS: [[BitBoard; 64]; 2] = [pawn_attacks(White), pawn_attacks(Black)];

/// Pre-compute pawn attack bitboards for each accessible square on the board
const fn pawn_attacks(colour: PieceColour) -> [BitBoard; 64] {
    let mut attacks = [BitBoard::EMPTY; 64];

    // Pawns cannot exist on first or last row but data is still needed
    // for hypothetical pawns for in_check checking
    let mut idx = 0;
    while idx < 64 {
        let row = idx as i8 / 8;
        let col = idx as i8 % 8;

        let target_row = match colour {
            White => row + 1,
            Black => row - 1,
        };

        if target_row >= 0 && target_row < 8 {
            let mut bb = BitBoard::EMPTY;
            if col > 0 {
                // Attack left
                bb.set_idx((target_row * 8 + col - 1) as usize);
            }
            if col < 7 {
                // Attack right
                bb.set_idx((target_row * 8 + col + 1) as usize);
            }
            attacks[idx] = bb;
        }

        idx += 1;
    }

    attacks
}

/// Checking if a square is attacked can be done cheap if the roles are reversed
/// A square is attacked by an enemy pawn if a hypothetical friendly pawn on the
/// check square would attack any enemy pawn
/// pos: square to check
/// enemy: the pawns that might or might not attack the target square
/// colour: colour of the friendly side
#[inline(always)]
pub fn is_square_attacked(pos: Square, enemy_pawns: BitBoard, colour: PieceColour) -> bool {
    get_attacks_pawn(pos, colour) & enemy_pawns != BitBoard::EMPTY
}

#[inline(always)]
pub fn get_attacks_pawn(pos: Square, colour: PieceColour) -> BitBoard {
    PAWN_ATTACKS[colour as usize][pos.idx()]
}

pub fn get_pl_moves_pawn(
    moves: &mut MoveList,
    pos: Square,
    friendly: BitBoard,
    enemy: BitBoard,
    colour: PieceColour,
    ep_square: Option<Square>,
) {
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

    // Attacks
    let ep_bb = match ep_square {
        None => BitBoard::EMPTY,
        Some(s) => {
            let mut bb = BitBoard::EMPTY;
            bb.set(s);
            bb
        }
    };
    for attacked_square in (PAWN_ATTACKS[colour as usize][pos.idx()] & (enemy | ep_bb)).iter() {
        if Some(attacked_square) == ep_square {
            moves.push(Move::new(EnPassant, pos, attacked_square));
        } else if (1..7).contains(&attacked_square.row()) {
            moves.push(Move::new(Capture, pos, attacked_square));
        } else {
            for variant in CAPTURE_PROMOTION_VARIANTS {
                moves.push(Move::new(variant, pos, attacked_square));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::{BitBoard, Square};
    use crate::chess::moves::MoveKind::{
        BishopPromotion, BishopPromotionCapture, Capture, DoublePawnPush, KnightPromotion,
        KnightPromotionCapture, QueenPromotion, QueenPromotionCapture, QuietMove, RookPromotion,
        RookPromotionCapture,
    };
    use crate::chess::moves::{Move, MoveList};
    use crate::chess::piece::PieceColour;
    use crate::chess::piece::PieceColour::{Black, White};
    use crate::move_gen::pawn::get_pl_moves_pawn;
    use std::str::FromStr;

    struct PawnTestCase {
        pos: &'static str,
        colour: PieceColour,
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

        let mut got = MoveList::new();
        get_pl_moves_pawn(&mut got, pos, friendly, enemy, case.colour, ep);
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
            colour: White,
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
            colour: White,
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
            colour: White,
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
            colour: White,
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
            colour: White,
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
            colour: White,
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
            colour: Black,
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
            colour: White,
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
            colour: Black,
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
            colour: White,
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
            colour: Black,
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

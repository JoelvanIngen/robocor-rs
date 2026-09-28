use crate::board::square::Square;

/// https://chessprogramming.org/Encoding_Moves
/// Bits: promotion, capture, special 1, special 2
#[repr(u8)]
pub enum MoveKind {
    QuietMove = 0b0000,
    DoublePawnPush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EnPassant = 0b0101,
    KnightPromotion = 0b1000,
    BishopPromotion = 0b1001,
    RookPromotion = 0b1010,
    QueenPromotion = 0b1011,
    KnightPromotionCapture = 0b1100,
    BishopPromotionCapture = 0b1101,
    RookPromotionCapture = 0b1110,
    QueenPromotionCapture = 0b1111,
}

/// TODO: Optimise into single u16 (4 bits for MoveKind, 6 for `from`, 6 for `to`)
pub struct Move {
    kind: MoveKind,
    from: Square,
    to: Square,
}

impl Move {
    pub fn new(kind: MoveKind, from: Square, to: Square) -> Self {
        Self { kind, from, to }
    }
}

use crate::board::square::Square;
use arrayvec::ArrayVec;
use std::fmt::Debug;

// Most achievable moves should be 218, and even that will never realistically happen
const MAX_MOVES: usize = 256;

pub type MoveList = ArrayVec<Move, MAX_MOVES>;

/// https://chessprogramming.org/Encoding_Moves
/// Bits: promotion, capture, special 1, special 2
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl MoveKind {
    pub fn from(v: u8) -> MoveKind {
        match v {
            0b0000 => MoveKind::QuietMove,
            0b0001 => MoveKind::DoublePawnPush,
            0b0010 => MoveKind::KingCastle,
            0b0011 => MoveKind::QueenCastle,
            0b0100 => MoveKind::Capture,
            0b0101 => MoveKind::EnPassant,
            0b1000 => MoveKind::KnightPromotion,
            0b1001 => MoveKind::BishopPromotion,
            0b1010 => MoveKind::RookPromotion,
            0b1011 => MoveKind::QueenPromotion,
            0b1100 => MoveKind::KnightPromotionCapture,
            0b1101 => MoveKind::BishopPromotionCapture,
            0b1110 => MoveKind::RookPromotionCapture,
            0b1111 => MoveKind::QueenPromotionCapture,
            _ => unreachable!(),
        }
    }
}

/// 4 bits for MoveKind, 6 for `from`, 6 for `to`
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move {
    repr: u16,
}

impl Move {
    pub fn new(kind: MoveKind, from: Square, to: Square) -> Self {
        Self {
            repr: ((kind as u16) << 12) + ((from.idx() as u16) << 6) + to.idx() as u16,
        }
    }

    pub fn kind(&self) -> MoveKind {
        MoveKind::from((self.repr >> 12) as u8)
    }

    pub fn from(&self) -> Square {
        Square::from_index(((self.repr >> 6) & 0b111_111) as usize)
    }

    pub fn to(&self) -> Square {
        Square::from_index((self.repr & 0b111_111) as usize)
    }

    pub fn is_capture(&self) -> bool {
        self.repr >> 12 & 0b0100 != 0
    }

    pub fn is_irreversable(&self) -> bool {
        // Only a quiet move is reversible
        self.repr >> 12 != 0b0000
    }

    pub fn is_promotion(&self) -> bool {
        self.repr >> 12 & 0b1000 != 0
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{} ({:?})", self.from(), self.to(), self.kind())
    }
}

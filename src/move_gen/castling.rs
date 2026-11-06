use crate::board::BitBoard;

/// Mask to cheaply remove castling rights when any piece moves to or from king/rook squares
pub const CASTLING_MASK: [BitBoard; 64] = {
    let mut masks = [BitBoard::from_u64(u64::MAX); 64];

    // White queenside
    masks[0] = BitBoard::from_u64(!(1u64 << 0));
    // White both sides
    masks[4] = BitBoard::from_u64(!((1u64 << 0) | (1u64 << 7)));
    // White kingside
    masks[7] = BitBoard::from_u64(!(1u64 << 7));

    // Black queenside
    masks[56] = BitBoard::from_u64(!(1u64 << 56));
    // Black both sides
    masks[60] = BitBoard::from_u64(!((1u64 << 56) | (1u64 << 63)));
    // Black kingside
    masks[63] = BitBoard::from_u64(!(1u64 << 63));

    masks
};

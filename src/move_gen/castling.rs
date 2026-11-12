/// Mask to cheaply remove castling rights when any piece moves to or from king/rook squares
pub const CASTLING_MASK: [u8; 64] = {
    let mut masks = [15u8; 64];

    // White queenside
    masks[0] = 15 - (1 << 3);
    // White both sides
    masks[4] = 15 - (1 << 3) - (1 << 2);
    // White kingside
    masks[7] = 15 - (1 << 2);

    // Black queenside
    masks[56] = 15 - (1 << 1);
    // Black both sides
    masks[60] = 15 - (1 << 1) - 1;
    // Black kingside
    masks[63] = 15 - 1;

    masks
};

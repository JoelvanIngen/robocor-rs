//! Late Move Reduction
//! https://chessprogramming.org/Late_Move_Reductions

use std::sync::LazyLock;

static LMR_TABLE: LazyLock<[[u8; 64]; 64]> = LazyLock::new(|| {
    let mut table = [[0; 64]; 64];
    for depth in 1..64 {
        for move_idx in 1..64 {
            // Made this formula up on the spot, inspiration from chessprogramming examples
            let r = 0.75 + (depth as f64).ln() * (move_idx as f64).ln() / 2.5;
            table[depth][move_idx] = r.floor() as u8;
        }
    }
    table
});

pub fn lmr_reduction(depth: u8, move_idx: usize) -> u8 {
    let d = (depth as usize).min(63);
    let m = move_idx.min(63);
    LMR_TABLE[d][m]
}

pub const MAGIC_SHIFT: usize = 52;

/// Finds and prints copy-pasteable magics for all squares
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;
    use crate::chess::zobrist::splitmix64;
    use crate::move_gen::bishop::{BISHOP_MAGICS, relevant_mask_bishop, slow_bishop_attacks};
    use crate::move_gen::rook::{ROOK_MAGICS, relevant_mask_rook, slow_rook_attacks};
    use std::collections::HashMap;

    #[test]
    #[ignore]
    fn print_rook_magics() {
        let mut output = String::from("const ROOK_MAGICS: [u64; 64] = [\n");
        for square_idx in 0..64usize {
            // Seed determined by keysmash
            let (magic, _) = find_rook_magic(square_idx, 0x6b2ca8852d224aaf ^ (square_idx as u64));
            output.push_str(&format!(
                "    0x{:016x}, // {}\n",
                magic,
                Square::from_index(square_idx)
            ));
        }
        output.push_str("];\n");
        println!("{}", output);
    }

    /// Finds magic multiplier for one square
    #[cfg(test)]
    fn find_rook_magic(square_idx: usize, seed: u64) -> (u64, u32) {
        let mask = relevant_mask_rook(square_idx);
        // Max 10 occupied for rook (excluding edges) and 9 for bishop
        // Therefore 54 could probably work too, but 52 is what I
        // find used as standard across other chess engines
        // TODO(Opt): increase to 54 to attempt to narrow the
        //            domain to reduce memory footprint
        let shift = MAGIC_SHIFT;

        let mut state = seed;
        loop {
            // Sparse bits find collision-free mapping more often than a random u64
            // (where approx half the bits should be set), so we AND 3 random numbers
            // Should result in about 1/8 bits being set?
            let (new_state, a) = splitmix64(state);
            state = new_state;
            let (new_state, b) = splitmix64(state);
            state = new_state;
            let (new_state, c) = splitmix64(state);
            state = new_state;

            let magic = a & b & c;

            // Track collisions
            let mut seen = HashMap::<usize, u64>::new();
            let mut ok = true;

            let mut subset = mask;
            loop {
                // Index that would be computed for this occupancy
                // Always < 4096 because subset is subset of mask
                // and popcnt(mask) <= 12, so shifting yields a
                // 12-bit value
                let idx = ((subset.wrapping_mul(magic)) >> shift) as usize;

                let attacks = slow_rook_attacks(square_idx, subset);

                match seen.get(&idx) {
                    // Collision with different attack set
                    Some(&prev) if prev != attacks => {
                        ok = false;
                        break;
                    }
                    // Either first use of this index or a duplicate which is also fine
                    _ => {
                        seen.insert(idx, attacks);
                    }
                }

                if subset == 0 {
                    break;
                }
                subset = (subset - 1) & mask;
            }

            if ok {
                return (magic, MAGIC_SHIFT as u32);
            }
        }
    }

    #[test]
    #[ignore]
    fn print_bishop_magics() {
        let mut output = String::from("const BISHOP_MAGICS: [u64; 64] = [\n");
        for square_idx in 0..64usize {
            // Seed determined by keysmash
            let (magic, _) =
                find_bishop_magic(square_idx, 0x6b2ca8852d224aaf ^ (square_idx as u64));
            output.push_str(&format!(
                "    0x{:016x}, // {}\n",
                magic,
                Square::from_index(square_idx)
            ));
        }
        output.push_str("];\n");
        println!("{}", output);
    }

    /// Finds magic multiplier for one square
    #[cfg(test)]
    fn find_bishop_magic(square_idx: usize, seed: u64) -> (u64, u32) {
        let mask = relevant_mask_bishop(square_idx);
        // Max 10 occupied for bishop (excluding edges) and 9 for bishop
        // Therefore 54 could probably work too, but 52 is what I
        // find used as standard across other chess engines
        // TODO(Opt): increase to 54 to attempt to narrow the
        //            domain to reduce memory footprint
        let shift = MAGIC_SHIFT;

        let mut state = seed;
        loop {
            // Sparse bits find collision-free mapping more often than a random u64
            // (where approx half the bits should be set), so we AND 3 random numbers
            // Should result in about 1/8 bits being set?
            let (new_state, a) = splitmix64(state);
            state = new_state;
            let (new_state, b) = splitmix64(state);
            state = new_state;
            let (new_state, c) = splitmix64(state);
            state = new_state;

            let magic = a & b & c;

            // Track collisions
            let mut seen = HashMap::<usize, u64>::new();
            let mut ok = true;

            let mut subset = mask;
            loop {
                // Index that would be computed for this occupancy
                // Always < 4096 because subset is subset of mask
                // and popcnt(mask) <= 12, so shifting yields a
                // 12-bit value
                let idx = ((subset.wrapping_mul(magic)) >> shift) as usize;

                let attacks = slow_bishop_attacks(square_idx, subset);

                match seen.get(&idx) {
                    // Collision with different attack set
                    Some(&prev) if prev != attacks => {
                        ok = false;
                        break;
                    }
                    // Either first use of this index or a duplicate which is also fine
                    _ => {
                        seen.insert(idx, attacks);
                    }
                }

                if subset == 0 {
                    break;
                }
                subset = (subset - 1) & mask;
            }

            if ok {
                return (magic, MAGIC_SHIFT as u32);
            }
        }
    }

    #[test]
    fn rooks_collision_free() {
        for sq in 0..64 {
            let mask = relevant_mask_rook(sq);
            let magic = ROOK_MAGICS[sq];
            let mut seen = HashMap::<usize, u64>::new();

            let mut subset = mask;
            loop {
                let idx = (subset.wrapping_mul(magic) >> MAGIC_SHIFT) as usize;
                let attacks = slow_rook_attacks(sq, subset);
                if let Some(&prev) = seen.get(&idx) {
                    assert_eq!(
                        prev,
                        attacks,
                        "MAGICS[{sq}] (square {}) collides: subset {subset:#x}",
                        Square::from_index(sq),
                    );
                } else {
                    seen.insert(idx, attacks);
                }
                if subset == 0 {
                    break;
                }
                subset = (subset - 1) & mask;
            }
        }
    }

    #[test]
    fn bishops_collision_free() {
        for sq in 0..64 {
            let mask = relevant_mask_bishop(sq);
            let magic = BISHOP_MAGICS[sq];
            let mut seen = HashMap::<usize, u64>::new();

            let mut subset = mask;
            loop {
                let idx = (subset.wrapping_mul(magic) >> MAGIC_SHIFT) as usize;
                let attacks = slow_bishop_attacks(sq, subset);
                if let Some(&prev) = seen.get(&idx) {
                    assert_eq!(
                        prev,
                        attacks,
                        "MAGICS[{sq}] (square {}) collides: subset {subset:#x}",
                        Square::from_index(sq),
                    );
                } else {
                    seen.insert(idx, attacks);
                }
                if subset == 0 {
                    break;
                }
                subset = (subset - 1) & mask;
            }
        }
    }
}

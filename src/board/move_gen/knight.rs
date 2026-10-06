use crate::board::bitboard::BitBoard;
use crate::board::moves::Move;
use crate::board::moves::MoveKind::{Capture, QuietMove};
use crate::board::square::Square;

const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (-2, 1),
    (-2, -1),
    (-1, 2),
    (-1, -2),
    (1, 2),
    (1, -2),
    (2, -1),
    (2, 1),
];

pub const KNIGHT_MOVES: [BitBoard; 64] = pre_compute_knight_moves();

/// Precomputes bitboards containing all possible moves for a knight from each position
const fn pre_compute_knight_moves() -> [BitBoard; 64] {
    let mut bitboards = [BitBoard::EMPTY; 64];

    let mut sq = 0usize;
    while sq < 64 {
        let row = sq / 8;
        let col = sq % 8;

        let mut moves = 0u64; // Bitboard

        let mut i = 0usize;
        while i < 8 {
            let delta = KNIGHT_DELTAS[i];
            let new_pos = (row as i8 + delta.0, col as i8 + delta.1);
            if new_pos.0 >= 0 && new_pos.1 >= 0 && new_pos.0 < 8 && new_pos.1 < 8 {
                moves |= 1 << (new_pos.0 * 8 + new_pos.1) as usize;
            }

            i += 1;
        }

        bitboards[sq] = BitBoard::from_u64(moves);

        sq += 1;
    }

    bitboards
}

pub fn get_pl_moves_knight(pos: Square, friendly: BitBoard, enemy: BitBoard) -> Vec<Move> {
    // Find potential from_coords knight squares
    let new_positions_bb = KNIGHT_MOVES[pos.idx()];
    let mut new_positions = Vec::new();

    for new_pos in new_positions_bb.iter() {
        if friendly.is_set(new_pos) {
            continue;
        }

        let move_type = match enemy.is_set(new_pos) {
            true => Capture,
            false => QuietMove,
        };

        new_positions.push(Move::new(move_type, pos, new_pos));
    }

    new_positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a1_empty() {
        let want = vec![
            Move::new(
                QuietMove,
                Square::from_coords(0, 0),
                Square::from_coords(2, 1),
            ),
            Move::new(
                QuietMove,
                Square::from_coords(0, 0),
                Square::from_coords(1, 2),
            ),
        ];
        let got = get_pl_moves_knight(Square::from_coords(0, 0), BitBoard::EMPTY, BitBoard::EMPTY);

        assert_eq!(
            want.len(),
            got.len(),
            "move count mismatch: {got:?} vs {want:?}"
        );
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }

    #[test]
    fn test_a1_friendly_targets() {
        let mut friendly = BitBoard::EMPTY;
        friendly.set(Square::from_coords(1, 2));
        friendly.set(Square::from_coords(2, 1));

        let moves = get_pl_moves_knight(Square::from_coords(0, 0), friendly, BitBoard::EMPTY);
        assert_eq!(moves.len(), 0, "is moving through friendly pieces");
    }

    #[test]
    fn test_a1_enemy_targets() {
        let want = vec![
            Move::new(
                Capture,
                Square::from_coords(0, 0),
                Square::from_coords(2, 1),
            ),
            Move::new(
                Capture,
                Square::from_coords(0, 0),
                Square::from_coords(1, 2),
            ),
        ];
        let mut enemy = BitBoard::EMPTY;
        enemy.set(Square::from_coords(1, 2));
        enemy.set(Square::from_coords(2, 1));

        let got = get_pl_moves_knight(Square::from_coords(0, 0), BitBoard::EMPTY, enemy);

        assert_eq!(
            want.len(),
            got.len(),
            "move count mismatch: {got:?} vs {want:?}"
        );
        for m in want {
            assert!(got.contains(&m), "missing move {m:?} in {got:?}");
        }
    }
}

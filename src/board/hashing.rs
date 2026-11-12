use crate::board::{BitBoard, Board};
use crate::chess::piece::PieceColour::Black;
use crate::chess::zobrist::{ZOBRIST, castling_index};

impl Board {
    #[inline(always)]
    pub(super) fn update_hash(&mut self) {
        self.hash ^= ZOBRIST.castling_rights[castling_index(self.castling_rights)];
        if self.en_passant != BitBoard::EMPTY {
            self.hash ^= ZOBRIST.en_passant[self.en_passant.get_square().col()];
        }
        if self.turn == Black {
            self.hash ^= ZOBRIST.side_to_move;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::zobrist::calculate_hash;

    /// Walks all possible moves within a certain depth
    /// When applying moves, the board will incrementally update its hash
    /// We then do a full recompute of the hash and check they stay equal
    fn walk_and_check_hash(board: Board, depth: u8) {
        let recomputed_hash = calculate_hash(
            board.bitboards,
            board.turn,
            board.castling_rights,
            board.en_passant,
        );
        assert_eq!(
            recomputed_hash, board.hash,
            "mismatch between recomputed hash and incremental hash"
        );

        if depth == 0 {
            return;
        }
        for m in board.generate_moves() {
            let mut new_board = board;
            new_board.apply_move(m);
            walk_and_check_hash(new_board, depth - 1);
        }
    }

    #[test]

    fn incremental_hash_matches_from_fen() {
        for (fen, depth) in [
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                3,
            ),
            (
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                3,
            ),
            ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 3),
            (
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                3,
            ),
            (
                "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
                3,
            ),
            (
                "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                3,
            ),
            (
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
                3,
            ),
        ] {
            let board = Board::from_fen(fen).unwrap();
            walk_and_check_hash(board, depth);
        }
    }
}

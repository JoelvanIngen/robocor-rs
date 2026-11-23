pub struct RepetitionTable {
    hashes: Vec<u64>,
    root_len: usize,      // Where search starts
    null_boundary: usize, // Index at which null moves began
}

impl RepetitionTable {
    pub fn new(mut history: Vec<u64>, root_hash: u64) -> Self {
        history.push(root_hash);
        let root_len = history.len();
        Self {
            hashes: history,
            root_len,
            null_boundary: 0,
        }
    }

    pub fn push(&mut self, hash: u64) {
        self.hashes.push(hash);
    }

    pub fn pop(&mut self) {
        self.hashes.pop();
    }

    // Pushes null move, and returns when the previous null move stack began
    pub fn push_null(&mut self, hash: u64) -> usize {
        let prev = self.null_boundary;
        self.push(hash);
        self.null_boundary = self.hashes.len();
        prev
    }

    pub fn pop_null(&mut self, prev_boundary: usize) {
        self.pop();
        self.null_boundary = prev_boundary;
    }

    pub fn is_repetition(&self, halfmove_clock: u8) -> bool {
        let len = self.hashes.len();
        if len < 3 {
            return false;
        }

        let current_hash = self.hashes[len - 1];

        // Moves that reset halfmove clock are irreversible,
        // and can therefore not be replicated
        // We can use this as a lower bound
        let min_idx = len
            .saturating_sub(halfmove_clock as usize + 1)
            .max(self.null_boundary);

        let mut count = 0;

        // First move that can be a replica is this move index - (2 plies)
        // Only consider own positions as opponent moves cannot create duplicates
        // of our current position
        let mut i = len - 3;
        loop {
            if i < min_idx {
                break;
            }
            if self.hashes[i] == current_hash {
                if i > self.root_len {
                    // Return true if duplicate in search to save computation
                    return true;
                }

                count += 1;
                if count >= 2 {
                    return true;
                }
            }

            if i < 2 {
                break;
            }
            i -= 2;
        }

        false
    }
}

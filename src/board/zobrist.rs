use std::sync::Arc;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

// 256 bit (8 bits x 32) seed (seed idea taken from rustic)
const RNG_SEED: [u8; 32] = [93; 32];

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ZobristValue {
    pub hash: u64,
    pub prev: Option<Arc<ZobristValue>>,
}
impl ZobristValue {
    pub fn empty() -> ZobristValue {
        ZobristValue {
            hash: 0,
            prev: None,
        }
    }
}

// Random values to use as base for Zobrist hashing
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ZobristHashes {
    // One random number for every square, piece and side
    pieces: [[[u64; 64]; 6]; 2],
    // One for every possible castle flag value (2x2x2x2)
    castling: [u64; 16],
    // One for every side
    side: [u64; 2],
    // One for every enpassant square + none case
    enpassant: [u64; 65],
}
impl ZobristHashes {
    pub fn init() -> ZobristHashes {
        let mut random = ChaChaRng::from_seed(RNG_SEED);
        ZobristHashes {
            pieces: [[[0; 64]; 6]; 2]
                .map(|side| side.map(|piece| piece.map(|_square| random.gen::<u64>()))),
            castling: [0; 16].map(|_castling| random.gen::<u64>()),
            side: [0; 2].map(|_side| random.gen::<u64>()),
            enpassant: [0; 65].map(|_eps| random.gen::<u64>()),
        }
    }

    pub fn piece(&self, side: usize, piece: usize, square: usize) -> ZobristValue {
        ZobristValue {
            hash: self.pieces[side][piece][square],
            prev: None,
        }
    }

    pub fn castling(&self, castling_permissions: usize) -> ZobristValue {
        ZobristValue {
            hash: self.castling[castling_permissions],
            prev: None,
        }
    }

    pub fn side(&self, side: usize) -> ZobristValue {
        ZobristValue {
            hash: self.side[side],
            prev: None,
        }
    }

    pub fn en_passant(&self, en_passant: usize) -> ZobristValue {
        ZobristValue {
            hash: self.enpassant[en_passant],
            prev: None,
        }
    }
}

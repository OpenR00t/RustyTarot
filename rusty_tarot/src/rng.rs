// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! Small xorshift RNG — no dependencies needed for shuffling a deck.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng(u64);

impl Rng {
    pub fn seeded() -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678);
        Self::from_seed(t)
    }

    pub fn from_seed(seed: u64) -> Self {
        let mut r = Rng((seed ^ 0x9E37_79B9_7F4A_7C15) | 1);
        for _ in 0..8 {
            r.next_u64();
        }
        r
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

/// Cheap deterministic hash for procedural sparkles.
pub fn hash2(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = x.wrapping_mul(374_761_393)
        ^ y.wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

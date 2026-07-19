/// Minimal random source used by deterministic gameplay code.
pub trait RandomSource {
    fn next_u64(&mut self) -> u64;

    fn next_f32(&mut self) -> f32 {
        const SCALE: f32 = 1.0 / ((1u32 << 24) as f32);
        ((self.next_u64() >> 40) as u32) as f32 * SCALE
    }

    fn range_u32_inclusive(&mut self, min: u32, max: u32) -> u32 {
        assert!(min <= max, "random range must not be empty");
        let span = max as u64 - min as u64 + 1;
        let accepted = u64::MAX - u64::MAX % span;
        loop {
            let value = self.next_u64();
            if value < accepted {
                return min + (value % span) as u32;
            }
        }
    }
}

/// SplitMix64 with a fixed algorithm, suitable for reproducible simulation streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl RandomSource for DeterministicRng {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_seeds_produce_equal_streams() {
        let mut left = DeterministicRng::new(42);
        let mut right = DeterministicRng::new(42);
        for _ in 0..32 {
            assert_eq!(left.next_u64(), right.next_u64());
        }
    }

    #[test]
    fn inclusive_ranges_stay_within_bounds() {
        let mut rng = DeterministicRng::new(7);
        for _ in 0..100 {
            assert!((3..=9).contains(&rng.range_u32_inclusive(3, 9)));
        }
    }
}

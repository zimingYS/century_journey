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

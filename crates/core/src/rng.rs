//! 世界确定性随机数

/// SplitMix64 风格的 64 位混合
#[inline]
fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

/// 世界确定性随机数生成器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldRng {
    seed: u64,
}
impl WorldRng {
    /// 新建
    #[inline]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 主种子
    #[inline]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// 逐次混合 salt 与坐标
    #[inline]
    fn hash_values(&self, salt: u64, values: &[i32]) -> u64 {
        let mut hash = mix(self.seed);
        hash = mix(hash ^ salt);
        for &value in values {
            hash = mix(hash ^ value as u64);
        }
        hash
    }

    /// 二维位置哈希
    #[inline]
    pub fn hash2(&self, salt: u64, x: i32, z: i32) -> u64 {
        self.hash_values(salt, &[x, z])
    }

    /// 三维位置哈希
    #[inline]
    pub fn hash3(&self, salt: u64, x: i32, y: i32, z: i32) -> u64 {
        self.hash_values(salt, &[x, y, z])
    }

    /// 二维哈希到 `[0, 1)`
    #[inline]
    pub fn unit2(&self, salt: u64, x: i32, z: i32) -> f32 {
        const SCALE: f32 = 1.0 / 16_777_216.0; // 2^24
        let hash = self.hash2(salt, x, z);
        let top_24 = (hash >> 40) as u32;
        top_24 as f32 * SCALE
    }

    /// 三维哈希到 `[0, 1)`
    #[inline]
    pub fn unit3(&self, salt: u64, x: i32, y: i32, z: i32) -> f32 {
        const SCALE: f32 = 1.0 / 16_777_216.0; // 2^24

        let hash = self.hash3(salt, x, y, z);
        let top_24 = (hash >> 40) as u32;

        top_24 as f32 * SCALE
    }

    /// 三维哈希到 `[0, n)`
    #[inline]
    pub fn range_u32(&self, salt: u64, x: i32, y: i32, z: i32, n: u32) -> u32 {
        debug_assert!(n > 0, "range_u32 requires n > 0");
        self.hash3(salt, x, y, z) as u32 % n
    }

    /// 确定性概率判定
    #[inline]
    pub fn chance(&self, salt: u64, x: i32, y: i32, z: i32, p: f32) -> bool {
        debug_assert!(
            (0.0..=1.0).contains(&p),
            "chance probability must be in [0, 1], got {p}"
        );
        if p <= 0.0 {
            return false;
        }
        if p >= 1.0 {
            return true;
        }
        let hash = self.hash3(salt, x, y, z);
        let top_24 = (hash >> 40) as u32;
        (top_24 as f32 / 16_777_216.0) < p
    }

    /// 从主种子派生子种子
    #[inline]
    pub fn derive_seed(&self, salt: u64) -> u64 {
        mix(mix(self.seed) ^ salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_produce_same_hash() {
        let a = WorldRng::new(12345);
        let b = WorldRng::new(12345);
        assert_eq!(a.hash2(7, -3, 9), b.hash2(7, -3, 9));
        assert_eq!(a.hash3(7, -3, 9, 11), b.hash3(7, -3, 9, 11));
    }

    #[test]
    fn different_seeds_diverge() {
        let a = WorldRng::new(1);
        let b = WorldRng::new(2);
        assert_ne!(
            a.hash2(0, 0, 0),
            b.hash2(0, 0, 0),
            "different seeds must not share a coordinate stream"
        );
    }

    #[test]
    fn coordinate_order_matters() {
        let rng = WorldRng::new(42);
        assert_ne!(rng.hash2(0, 3, 5), rng.hash2(0, 5, 3));
    }

    #[test]
    fn salt_separates_channels() {
        let rng = WorldRng::new(42);
        let mut seen = std::collections::HashSet::new();
        for salt in 0..64u64 {
            assert!(
                seen.insert(rng.hash2(salt, 10, 20)),
                "salt {salt} collided with an earlier salt"
            );
        }
    }

    #[test]
    fn negative_coordinates_ok() {
        let rng = WorldRng::new(99);
        let mut seen = std::collections::HashSet::new();
        for x in -50..50 {
            for z in -50..50 {
                assert!(seen.insert(rng.hash2(0, x, z)), "collision at ({x}, {z})");
            }
        }
    }

    #[test]
    fn unit2_in_unit_range() {
        let rng = WorldRng::new(2024);
        for x in -20..20 {
            for z in -20..20 {
                let v = rng.unit2(0, x, z);
                assert!((0.0..1.0).contains(&v), "unit2 out of range: {v}");
            }
        }
    }

    #[test]
    fn unit2_average_near_half() {
        let rng = WorldRng::new(7);
        let mut sum = 0.0_f64;
        let mut n = 0u64;
        for x in 0..100 {
            for z in 0..100 {
                sum += rng.unit2(0, x, z) as f64;
                n += 1;
            }
        }
        let mean = sum / n as f64;
        assert!((mean - 0.5).abs() < 0.02, "unit2 mean is skewed: {mean}");
    }

    #[test]
    fn unit3_in_unit_range() {
        let rng = WorldRng::new(2024);
        for y in -5..5 {
            let v = rng.unit3(0, 1, y, 2);
            assert!((0.0..1.0).contains(&v), "unit3 out of range: {v}");
        }
    }

    #[test]
    fn range_u32_respects_bound() {
        let rng = WorldRng::new(555);
        for x in 0..50 {
            let v = rng.range_u32(0, x, 0, 0, 7);
            assert!(v < 7, "range_u32 exceeded bound: {v}");
        }
    }

    #[test]
    fn range_u32_covers_all_buckets() {
        let rng = WorldRng::new(31337);
        let mut buckets = [0u32; 4];
        for x in 0..200 {
            let v = rng.range_u32(0, x, 0, 0, 4) as usize;
            buckets[v] += 1;
        }
        assert!(buckets.iter().all(|&c| c > 0), "buckets: {buckets:?}");
    }

    #[test]
    fn range_u32_n_one_always_zero() {
        let rng = WorldRng::new(1);
        for x in 0..20 {
            assert_eq!(rng.range_u32(0, x, 0, 0, 1), 0);
        }
    }

    #[test]
    fn chance_boundaries_are_certain() {
        let rng = WorldRng::new(88);
        for x in 0..50 {
            assert!(!rng.chance(0, x, 0, 0, 0.0), "p=0 must never hit");
            assert!(rng.chance(0, x, 0, 0, 1.0), "p=1 must always hit");
        }
    }

    #[test]
    fn chance_rate_is_roughly_p() {
        let rng = WorldRng::new(1234);
        let mut hits = 0u32;
        let total = 10_000u32;
        for x in 0..100 {
            for z in 0..100 {
                if rng.chance(0, x, 0, z, 0.25) {
                    hits += 1;
                }
            }
        }
        let rate = hits as f32 / total as f32;
        assert!((rate - 0.25).abs() < 0.02, "chance rate is skewed: {rate}");
    }

    #[test]
    fn chance_is_deterministic() {
        let a = WorldRng::new(777);
        let b = WorldRng::new(777);
        for x in 0..30 {
            assert_eq!(
                a.chance(3, x, 1, 2, 0.5),
                b.chance(3, x, 1, 2, 0.5),
                "chance must be reproducible at x={x}"
            );
        }
    }

    #[test]
    fn derive_seed_is_deterministic_and_salt_separated() {
        let rng = WorldRng::new(2026);

        assert_eq!(rng.derive_seed(1), WorldRng::new(2026).derive_seed(1));

        let mut seen = std::collections::HashSet::new();
        for salt in 0..64u64 {
            assert!(
                seen.insert(rng.derive_seed(salt)),
                "derive_seed collision at salt {salt}"
            );
        }

        assert_ne!(
            rng.derive_seed(0),
            rng.seed(),
            "derived seed must differ from the root seed"
        );
    }

    #[test]
    fn derive_seed_changes_coordinate_stream() {
        let root = WorldRng::new(2026);
        let child = WorldRng::new(root.derive_seed(9));
        assert_ne!(root.hash2(0, 1, 2), child.hash2(0, 1, 2));
    }
}

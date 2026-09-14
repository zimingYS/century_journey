//! 世界种子

use cj_core::rng::WorldRng;

/// 当前世界生成器版本（影响地形产物时递增）
pub const CURRENT_GENERATOR_VERSION: u32 = 1;

/// 世界种子随机数命名空间，隔离世界根种子与子系统 salt
const WORLD_SEED_NAMESPACE: u64 = 0x434A_0000_0000_0001;

/// 世界种子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldSeed {
    /// 玩家可见的世界种子
    seed: u64,
    /// 生成器版本
    generator_version: u32,
}

impl WorldSeed {
    /// 构造世界种子
    #[inline]
    pub const fn new(seed: u64, generator_version: u32) -> Self {
        Self {
            seed,
            generator_version,
        }
    }

    /// 使用当前生成器版本构造
    #[inline]
    pub const fn from_seed(seed: u64) -> Self {
        Self::new(seed, CURRENT_GENERATOR_VERSION)
    }

    /// 世界种子值
    #[inline]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// 生成器版本
    #[inline]
    pub const fn generator_version(&self) -> u32 {
        self.generator_version
    }

    /// 确定性随机源
    #[inline]
    pub const fn rng(&self) -> WorldRng {
        WorldRng::new(self.seed ^ WORLD_SEED_NAMESPACE)
    }

    /// 生成器版本是否为当前版本（仅相等判断）
    #[inline]
    pub const fn matches_current(&self) -> bool {
        self.generator_version == CURRENT_GENERATOR_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_seed_uses_current_version() {
        let seed = WorldSeed::from_seed(12345);
        assert_eq!(seed.seed(), 12345);
        assert_eq!(seed.generator_version(), CURRENT_GENERATOR_VERSION);
        assert!(seed.matches_current());
    }

    #[test]
    fn new_preserves_both_fields() {
        let seed = WorldSeed::new(999, 7);
        assert_eq!(seed.seed(), 999);
        assert_eq!(seed.generator_version(), 7);
    }

    #[test]
    fn matches_current_rejects_other_versions() {
        assert!(!WorldSeed::new(0, 0).matches_current());
        assert!(!WorldSeed::new(0, CURRENT_GENERATOR_VERSION + 1).matches_current());
        assert!(!WorldSeed::new(0, u32::MAX).matches_current());
    }

    #[test]
    fn rng_is_deterministic() {
        let a = WorldSeed::from_seed(2026).rng();
        let b = WorldSeed::from_seed(2026).rng();

        for x in 0..20 {
            assert_eq!(a.hash2(0, x, 0), b.hash2(0, x, 0));
        }
    }

    #[test]
    fn different_world_seeds_diverge() {
        let a = WorldSeed::from_seed(1).rng();
        let b = WorldSeed::from_seed(2).rng();
        assert_ne!(a.hash2(0, 0, 0), b.hash2(0, 0, 0));
    }

    #[test]
    fn rng_is_namespaced_away_from_raw_seed() {
        let world = WorldSeed::from_seed(42).rng();
        let raw = WorldRng::new(42);
        assert_ne!(
            world.hash2(0, 0, 0),
            raw.hash2(0, 0, 0),
            "rng() must not equal a raw seed stream"
        );
    }

    #[test]
    fn seed_value_is_stable_across_versions() {
        assert_eq!(WorldSeed::new(77, 1).seed(), WorldSeed::new(77, 2).seed());
        assert_ne!(
            WorldSeed::new(77, 1).generator_version(),
            WorldSeed::new(77, 2).generator_version()
        );
    }

    #[test]
    fn world_seed_equality_and_hash_follow_fields() {
        let a = WorldSeed::new(5, 1);
        let b = WorldSeed::new(5, 1);
        let c = WorldSeed::new(5, 2);

        assert_eq!(a, b);
        assert_ne!(a, c, "version difference must break equality");

        let mut set = std::collections::HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }
}

//! 世界确定性随机数。

/// SplitMix64 风格的 64 位混合函数。
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
    /// 创建一个新的世界随机数对象。
    #[inline]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 获取当前主种子。
    #[inline]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// 对输入进行逐次混合。
    #[inline]
    fn hash_values(&self, salt: u64, values: &[i32]) -> u64 {
        let mut hash = mix(self.seed);
        hash = mix(hash ^ salt);
        for &value in values {
            hash = mix(hash ^ value as u64);
        }
        hash
    }

    /// 计算二维位置哈希。
    #[inline]
    pub fn hash2(&self, salt: u64, x: i32, z: i32) -> u64 {
        self.hash_values(salt, &[x, z])
    }

    /// 计算三维位置哈希。
    #[inline]
    pub fn hash3(&self, salt: u64, x: i32, y: i32, z: i32) -> u64 {
        self.hash_values(salt, &[x, y, z])
    }

    /// 将二维位置哈希转换到 `[0, 1)`。
    #[inline]
    pub fn unit2(&self, salt: u64, x: i32, z: i32) -> f32 {
        const SCALE: f32 = 1.0 / 16_777_216.0; // 2^24
        let hash = self.hash2(salt, x, z);
        let top_24 = (hash >> 40) as u32;
        top_24 as f32 * SCALE
    }

    /// 将三维位置哈希转换为 `[0, 1)`
    #[inline]
    pub fn unit3(&self, salt: u64, x: i32, y: i32, z: i32) -> f32 {
        const SCALE: f32 = 1.0 / 16_777_216.0; // 2^24

        let hash = self.hash3(salt, x, y, z);
        let top_24 = (hash >> 40) as u32;

        top_24 as f32 * SCALE
    }

    /// 将三维位置哈希映射到 `[0, n)`。
    #[inline]
    pub fn range_u32(&self, salt: u64, x: i32, y: i32, z: i32, n: u32) -> u32 {
        debug_assert!(n > 0, "range_u32 requires n > 0");
        self.hash3(salt, x, y, z) as u32 % n
    }

    /// 进行一个确定性的概率判定。
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

    /// 从主种子派生一个新的子种子。
    #[inline]
    pub fn derive_seed(&self, salt: u64) -> u64 {
        mix(mix(self.seed) ^ salt)
    }
}

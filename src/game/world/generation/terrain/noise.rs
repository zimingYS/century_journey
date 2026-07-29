use noise::Perlin;

/// 多层噪声采样器
pub struct NoiseSampler {
    /// 种子
    pub seed: u32,
    /// 主地形噪声（大尺度起伏）
    pub terrain_primary: Perlin,
    /// 地形细节噪声（小尺度变化）
    pub terrain_detail: Perlin,
    /// 粗糙度噪声
    pub roughness: Perlin,
    /// 洞穴噪声
    pub cave: Perlin,
}

impl NoiseSampler {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            terrain_primary: Perlin::new(seed),
            terrain_detail: Perlin::new(seed.wrapping_add(100)),
            roughness: Perlin::new(seed.wrapping_add(200)),
            cave: Perlin::new(seed.wrapping_add(300)),
        }
    }
}

impl Clone for NoiseSampler {
    fn clone(&self) -> Self {
        Self::new(self.seed)
    }
}

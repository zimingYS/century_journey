//! 单区块光照数据的打包存储与访问。
//!
//! `ChunkLight` 是会话期世界状态（不进存档）：每体素 4 字节打包
//! 天空光 RGB 与方块光 RGB（每通道 4bit）。由后台传播任务写入，固定步提交，
//! 客户端网格构建通过快照消费。

use crate::game::world::chunk::ChunkData;
use crate::shared::voxel::CHUNK_VOLUME;

/// 单区块光照数据（会话期态，不进存档）。
///
/// 低 12bit 保存方块光 RGB，高 12bit 保存天空光 RGB。独立初始化标记用于
/// 区分“合法的全黑洞穴”和“尚未计算”，避免全零区块永远无法构建网格。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkLight {
    packed: Box<[u32; CHUNK_VOLUME]>,
    initialized: bool,
    fingerprint: u64,
}

impl Default for ChunkLight {
    fn default() -> Self {
        Self {
            packed: Box::new([0u32; CHUNK_VOLUME]),
            initialized: false,
            fingerprint: 0,
        }
    }
}

/// 三通道 4bit 光级。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct LightRgb {
    /// 红色通道（0-15）。
    pub r: u8,
    /// 绿色通道（0-15）。
    pub g: u8,
    /// 蓝色通道（0-15）。
    pub b: u8,
}

impl LightRgb {
    /// 从 0-15 强度和线性 RGB 颜色构造量化光级。
    pub fn from_emission(emission: u8, color: [f32; 3]) -> Self {
        let emission = emission.min(15) as f32;
        Self {
            r: (emission * color[0].clamp(0.0, 1.0)).round() as u8,
            g: (emission * color[1].clamp(0.0, 1.0)).round() as u8,
            b: (emission * color[2].clamp(0.0, 1.0)).round() as u8,
        }
    }

    /// 判断三个通道是否均为零。
    pub const fn is_dark(self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    /// 逐通道取最大值，用于不同路径和不同光源的确定性混色。
    pub fn max_assign(&mut self, other: Self) -> bool {
        let previous = *self;
        self.r = self.r.max(other.r);
        self.g = self.g.max(other.g);
        self.b = self.b.max(other.b);
        *self != previous
    }

    /// 逐通道应用透射滤色，并执行一格传播衰减。
    pub fn attenuated(self, filter: [f32; 3]) -> Self {
        let channel = |value: u8, transmission: f32| {
            (value.saturating_sub(1) as f32 * transmission.clamp(0.0, 1.0)).floor() as u8
        };
        Self {
            r: channel(self.r, filter[0]),
            g: channel(self.g, filter[1]),
            b: channel(self.b, filter[2]),
        }
    }

    /// 逐通道应用透射滤色，但不增加垂直直射天空光的距离衰减。
    pub fn filtered(self, filter: [f32; 3]) -> Self {
        let channel = |value: u8, transmission: f32| {
            (value as f32 * transmission.clamp(0.0, 1.0)).floor() as u8
        };
        Self {
            r: channel(self.r, filter[0]),
            g: channel(self.g, filter[1]),
            b: channel(self.b, filter[2]),
        }
    }
}

/// 单个体素的天空光与方块光。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct LightCell {
    /// 自然天空光；RGB 独立存储以支持染色玻璃。
    pub sky: LightRgb,
    /// 数据驱动方块光；多个光源按通道合成。
    pub block: LightRgb,
}

impl LightCell {
    /// 返回天空光与方块光逐通道合并后的表面光色。
    pub fn combined(self) -> LightRgb {
        LightRgb {
            r: self.sky.r.max(self.block.r),
            g: self.sky.g.max(self.block.g),
            b: self.sky.b.max(self.block.b),
        }
    }
}

impl ChunkLight {
    /// 读取区块局部坐标处的光级；调用方必须保证坐标位于区块范围内。
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> LightCell {
        let p = self.packed[ChunkData::xyz_to_index(x, y, z)];
        LightCell {
            sky: LightRgb {
                r: ((p >> 20) & 0xF) as u8,
                g: ((p >> 16) & 0xF) as u8,
                b: ((p >> 12) & 0xF) as u8,
            },
            block: LightRgb {
                r: ((p >> 8) & 0xF) as u8,
                g: ((p >> 4) & 0xF) as u8,
                b: (p & 0xF) as u8,
            },
        }
    }

    /// 写入区块局部坐标处的光级；调用方必须保证坐标位于区块范围内。
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, cell: LightCell) {
        let idx = ChunkData::xyz_to_index(x, y, z);
        self.packed[idx] = ((cell.sky.r.min(15) as u32) << 20)
            | ((cell.sky.g.min(15) as u32) << 16)
            | ((cell.sky.b.min(15) as u32) << 12)
            | ((cell.block.r.min(15) as u32) << 8)
            | ((cell.block.g.min(15) as u32) << 4)
            | cell.block.b.min(15) as u32;
        self.initialized = false;
        self.fingerprint = 0;
    }

    /// 清空全部通道并回到未初始化状态。
    pub fn reset(&mut self) {
        self.packed.fill(0);
        self.initialized = false;
        self.fingerprint = 0;
    }

    /// 标记本区块完成一次完整传播，并缓存主线程差异判断使用的摘要。
    pub fn mark_initialized(&mut self) {
        // FNV-1a 足以作为变化摘要；完整数组仍保留为权威数据，不参与每次提交比较。
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        for value in self.packed.iter() {
            fingerprint ^= u64::from(*value);
            fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.initialized = true;
        self.fingerprint = fingerprint;
    }

    /// 光数组是否完成过与当前世界修订对应的传播。
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 返回完整传播结束时生成的轻量变化摘要。
    #[inline]
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/chunk_light.rs"]
mod tests;

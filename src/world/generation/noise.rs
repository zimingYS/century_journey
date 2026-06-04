use crate::voxel::registry::BlockRegistry;
use crate::world::chunk::ChunkData;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use crate::core::constant::world::{CHUNK_SIZE, CHUNK_VOLUME, MAP_HEIGHT_SCALE, NOISE_SCALE, SEA_LEVEL};

/// 地形生成器
pub struct TerrainGenerator {
    perlin: Perlin,
}

/// 生成地形主要使用的方块缓存
#[derive(Clone, Copy)]
pub struct GenerationBlockIds {
    pub air: u16,
    pub grass: u16,
    pub dirt: u16,
    pub stone: u16,
    pub sand: u16,
    pub water: u16,
    pub snow: u16,
    pub leaves: u16,
    pub wood: u16,
}

impl GenerationBlockIds {
    /// 游戏在调用生成前，从 Bevy 的中央注册表中一次性把名字翻译成数字 ID
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        Self {
            air: 0,
            grass: registry.get_id_by_identifier("century_journey:grass").unwrap_or(0),
            dirt:  registry.get_id_by_identifier("century_journey:dirt").unwrap_or(0),
            stone: registry.get_id_by_identifier("century_journey:stone").unwrap_or(0),
            sand:  registry.get_id_by_identifier("century_journey:sand").unwrap_or(0),
            water: registry.get_id_by_identifier("century_journey:water").unwrap_or(0),
            snow:  registry.get_id_by_identifier("century_journey:snow").unwrap_or(0),
            leaves: registry.get_id_by_identifier("century_journey:leaves").unwrap_or(0),
            wood:  registry.get_id_by_identifier("century_journey:wood").unwrap_or(0),
        }
    }

    /// 从方块标识符解析到ID
    /// 缓存了常用方块，未缓存的回注册表查
    pub fn resolve_block_id(&self, _identifier: &str) -> u16 {
        // 这里用简单的匹配，因为 GenerationBlockIds 没有存 BlockRegistry 引用
        // 实际使用时，如果群系的方块不在缓存中，需要从 registry 获取
        match _identifier {
            "century_journey:grass" => self.grass,
            "century_journey:dirt" => self.dirt,
            "century_journey:stone" => self.stone,
            "century_journey:sand" => self.sand,
            "century_journey:water" => self.water,
            "century_journey:snow" => self.snow,
            "century_journey:leaves" => self.leaves,
            "century_journey:wood" => self.wood,
            _ => self.grass, // 未知方块回退到草方块
        }
    }
}

/// 缓存方块ID资源，避免每帧重建
#[derive(Resource, Clone, Copy)]
pub struct CachedBlockIds(pub GenerationBlockIds);

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
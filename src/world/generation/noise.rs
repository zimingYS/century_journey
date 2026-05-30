use crate::core::constant::{CHUNK_SIZE, CHUNK_VOLUME, MAP_HEIGHT_SCALE, NOISE_SCALE, SEA_LEVEL};
use crate::voxel::registry::BlockRegistry;
use crate::voxel::types::VoxelType;
use crate::world::chunk::ChunkData;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

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
        }
    }
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
        }
    }

    // 根据噪声生成方块数据
    pub fn generate_chunk(&self, chunk_post: IVec3, block_ids: GenerationBlockIds) -> ChunkData{
        let mut voxels = [0u16; CHUNK_VOLUME];
        let mut chunk_data = ChunkData {voxels};

        // 计算当前区块在世界中的坐标偏移量
        let world_start_x = chunk_post.x * CHUNK_SIZE as i32;
        let world_start_y = chunk_post.y * CHUNK_SIZE as i32;
        let world_start_z = chunk_post.z * CHUNK_SIZE as i32;

        // 遍历区块每个格子
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // 换算出该格子的绝对坐标
                let world_x = world_start_x + x as i32;
                let world_z = world_start_z + z as i32;

                // 根据世界坐标生成地形高度
                let noise_value = self.perlin.get([world_x as f64 * NOISE_SCALE, world_z as f64 * NOISE_SCALE]);
                // 将噪声值映射到世界Y轴高度
                let target_surface_y = ((noise_value * MAP_HEIGHT_SCALE) + SEA_LEVEL as f64) as i32;

                for y in 0..CHUNK_SIZE {
                    let world_y = world_start_y + y as i32;

                    // 根据方块缓存生成对应标签
                    let voxel_id = if world_y > target_surface_y {
                        if world_y <= SEA_LEVEL {
                            block_ids.water
                        } else {
                            block_ids.air
                        }
                    } else if world_y == target_surface_y {
                        if world_y <= SEA_LEVEL + 2 {
                            block_ids.sand
                        } else {
                            block_ids.grass
                        }
                    } else if world_y > target_surface_y - 4 {
                        if target_surface_y <= SEA_LEVEL {
                            block_ids.sand
                        } else {
                            block_ids.dirt
                        }
                    } else {
                        block_ids.stone
                    };

                    chunk_data.set_voxel(x, y, z, voxel_id);
                }
            }
        }

        chunk_data
    }
}
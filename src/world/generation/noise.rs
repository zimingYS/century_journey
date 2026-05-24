use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use crate::core::constant::{CHUNK_SIZE, CHUNK_VOLUME, MAP_HEIGHT_SCALE, NOISE_SCALE, SEA_LEVEL};
use crate::voxel;
use crate::voxel::types::VoxelType;
use crate::world::chunk::ChunkData;

pub struct TerrainGenerator {
    perlin: Perlin,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
        }
    }

    // 根据噪声生成方块数据
    pub fn generate_chunk(&self, chunk_post: IVec3) -> ChunkData{
        let mut voxels = [0u8; CHUNK_VOLUME];
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

                    let voxel_type = if world_y > target_surface_y {
                        if world_y <= SEA_LEVEL {
                            VoxelType::Water
                        } else {
                            VoxelType::Air
                        }
                    }else if world_y == target_surface_y{
                        if world_y <= SEA_LEVEL + 2 {
                            VoxelType::Sand
                        } else {
                            VoxelType::Grass
                        }
                    } else if world_y > target_surface_y - 4{
                        if target_surface_y <= SEA_LEVEL {
                            VoxelType::Sand  
                        } else {
                            VoxelType::Dirt
                        }
                    }else {
                        VoxelType::Stone
                    };

                    chunk_data.set_voxel(x, y, z, voxel_type);
                }
            }
        }

        chunk_data
    }
}
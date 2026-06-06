use bevy::prelude::IVec3;
use noise::NoiseFn;
use crate::core::constant::world::*;
use crate::world::chunk::ChunkData;
use crate::world::generation::biome::BiomeRegistry;
use crate::world::generation::climate::{ClimateSampler, Season};
use crate::world::generation::context::ChunkGenContext;
use crate::world::generation::noise::{GenerationBlockIds, NoiseSampler};

/// 地形生成器 — 根据群系参数生成地形
pub struct TerrainGenerator;

impl TerrainGenerator {
    /// 生成区块的气候/群系上下文
    pub fn sample_context(
        noise_sampler: &NoiseSampler,
        climate_sampler: &ClimateSampler,
        season: Season,
        biome_registry: &BiomeRegistry,
        chunk_pos: IVec3,
    ) -> ChunkGenContext {
        let world_start_x = chunk_pos.x * CHUNK_SIZE as i32;
        let world_start_z = chunk_pos.z * CHUNK_SIZE as i32;

        let mut ctx = ChunkGenContext::new(chunk_pos);

        // 扩展一圈以包含邻居边界，使平滑核能真正跨区块采样
        const PADDED: usize = CHUNK_SIZE + 2;
        let mut raw_heights = [[0.0f64; PADDED]; PADDED];

        let mut cached_temperature = [[0.0f64; PADDED]; PADDED];
        let mut cached_humidity = [[0.0f64; PADDED]; PADDED];
        
        for x in 0..PADDED {
            for z in 0..PADDED {
                // 偏移 -1 以覆盖区块边界外一圈的世界坐标
                let world_x = world_start_x + x as i32 - 1;
                let world_z = world_start_z + z as i32 - 1;

                // 采样气候
                let temperature = climate_sampler.sample_temperature_with_season(world_x, world_z, season);
                let humidity = climate_sampler.sample_humidity_with_season(world_x, world_z, season);
                cached_temperature[x][z] = temperature;
                cached_humidity[x][z] = humidity;

                let blended = biome_registry.blend_terrain_params(temperature, humidity);

                // 采样噪声（统一缩放，与群系无关）
                let primary = noise_sampler.terrain_primary.get([
                    world_x as f64 * GLOBAL_TERRAIN_SCALE,
                    world_z as f64 * GLOBAL_TERRAIN_SCALE,
                ]);
                let detail = noise_sampler.terrain_detail.get([
                    world_x as f64 * GLOBAL_DETAIL_SCALE,
                    world_z as f64 * GLOBAL_DETAIL_SCALE,
                ]);
                let rough = noise_sampler.roughness.get([
                    world_x as f64 * GLOBAL_ROUGHNESS_SCALE,
                    world_z as f64 * GLOBAL_ROUGHNESS_SCALE,
                ]);

                let roughness_factor = (rough + 1.0) * 0.5 * blended.roughness;
                let base_height = blended.base_height
                    + primary * blended.height_amplitude
                    + detail * blended.height_amplitude * 0.3 * roughness_factor;

                raw_heights[x][z] = base_height;
            }
        }

        let kernel = [
            [0.0625, 0.125, 0.0625],
            [0.125,  0.25,  0.125],
            [0.0625, 0.125, 0.0625],
        ];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = world_start_x + x as i32;
                let world_z = world_start_z + z as i32;

                let temperature = cached_temperature[x + 1][z + 1];
                let humidity = cached_humidity[x + 1][z + 1];

                // 主要群系（用于 surface_block、tree_density 等）
                let biome_index = biome_registry.select_biome(temperature, humidity);

                // 平滑后的高度（在 padded 数组中偏移 +1）
                let mut smoothed = 0.0;
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        let nx = (x + 1) as i32 + dx;
                        let nz = (z + 1) as i32 + dz;
                        smoothed += raw_heights[nx as usize][nz as usize]
                            * kernel[(dx + 1) as usize][(dz + 1) as usize];
                    }
                }

                // 平滑核现在已包含真实的跨区块邻居数据，无需 edge_factor 补偿
                let final_height: f64  = smoothed;

                ctx.columns.push(crate::world::generation::context::ColumnContext {
                    world_x,
                    world_z,
                    temperature,
                    humidity,
                    biome_index,
                    base_height: final_height.round() as i32,
                    roughness: 0.0,
                });
            }
        }

        ctx
    }

    /// 根据上下文填充方块数据
    pub fn generate_terrain(
        ctx: &ChunkGenContext,
        block_ids: &GenerationBlockIds,
        biome_registry: &BiomeRegistry,
    ) -> ChunkData {
        let mut voxels = [0u16; CHUNK_VOLUME];
        let mut chunk_data = ChunkData { voxels };
        let world_start_y = ctx.chunk_pos.y * CHUNK_SIZE as i32;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let col = ctx.get_column(x, z);
                let target_surface_y = col.base_height;
                let biome = biome_registry.get(col.biome_index).unwrap();

                let surface_id = block_ids.resolve_block_id(&biome.surface_block);
                let subsurface_id = block_ids.resolve_block_id(&biome.subsurface_block);
                let beach_id = block_ids.resolve_block_id(&biome.beach_block);

                for y in 0..CHUNK_SIZE {
                    let world_y = world_start_y + y as i32;

                    let voxel_id = if world_y > target_surface_y {
                        if world_y <= SEA_LEVEL {
                            block_ids.water
                        } else {
                            block_ids.air
                        }
                    } else if world_y == target_surface_y {
                        if world_y <= SEA_LEVEL + 2 {
                            beach_id
                        } else {
                            surface_id
                        }
                    } else if world_y > target_surface_y - 4 {
                        if target_surface_y <= SEA_LEVEL {
                            beach_id
                        } else {
                            subsurface_id
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
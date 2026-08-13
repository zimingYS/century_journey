//! 使用地形上下文生成基础体素层和地表覆盖。

use super::SEA_LEVEL;
use super::climate::ClimateSampler;
use super::constants::{GLOBAL_DETAIL_SCALE, GLOBAL_ROUGHNESS_SCALE, GLOBAL_TERRAIN_SCALE};
use super::context::{ChunkGenContext, ColumnContext};
use super::noise::NoiseSampler;
use crate::content::biome::BiomeRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::biome::classifier::{blend_terrain_params, select_biome};
use crate::game::world::generation::block_ids::GenerationBlockIds;
use crate::game::world::generation::pipeline::BaseGenerationKey;
use crate::shared::voxel::{CHUNK_SIZE, CHUNK_VOLUME};
use noise::NoiseFn;

/// 单个世界列的确定性地表采样结果。
///
/// `ground_height` 与基础区块生成写入的地表方块 Y 坐标一致。方块标识使用内容层的
/// 稳定 `Identifier`，供上层只读采样服务解析为当前内容注册表的运行时 ID。
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainSurfaceSample {
    /// 基础地表方块所在的世界 Y 坐标。
    pub ground_height: i32,
    /// 地表或海面可见顶面的世界 Y 坐标。
    ///
    /// 这个值已经包含体素顶面偏移和海平面覆盖，调用方不需要读取 Game 内部的
    /// 海平面常量即可还原真实可见顶面。
    pub visible_surface_height: i32,
    /// 当前列是否由海平面水体覆盖；仅用于远景着色，不代表可交互流体状态。
    pub is_water_surface: bool,
    /// 按基础区块生成规则选出的可见地表方块（草、沙等）。
    pub surface_block: crate::shared::identifier::Identifier,
    /// 按基础区块生成规则选出的次表方块（土、沙等）。
    pub subsurface_block: crate::shared::identifier::Identifier,
    /// 与当前生成版本一致的温度采样，供纯表现层着色使用。
    pub temperature: f64,
    /// 与当前生成版本一致的湿度采样，供纯表现层着色使用。
    pub humidity: f64,
    /// 按稳定注册顺序选出的主生物群系索引。
    pub biome_index: u8,
}

/// 区块生成和远景采样共用的 3x3 高度平滑核。
const SURFACE_SMOOTHING_KERNEL: [[f64; 3]; 3] = [
    [0.0625, 0.125, 0.0625],
    [0.125, 0.25, 0.125],
    [0.0625, 0.125, 0.0625],
];

fn raw_surface_sample(
    noise_sampler: &NoiseSampler,
    climate_sampler: &ClimateSampler,
    biome_registry: &BiomeRegistry,
    generation_version: u32,
    world_x: i32,
    world_z: i32,
) -> RawSurfaceSample {
    let temperature =
        climate_sampler.sample_generation_temperature(world_x, world_z, generation_version);
    let humidity = climate_sampler.sample_generation_humidity(world_x, world_z, generation_version);
    let blended = blend_terrain_params(biome_registry, temperature, humidity);
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

    RawSurfaceSample {
        temperature,
        humidity,
        height: blended.base_height
            + primary * blended.height_amplitude
            + detail * blended.height_amplitude * 0.3 * roughness_factor,
    }
}

fn smooth_surface_height(
    noise_sampler: &NoiseSampler,
    climate_sampler: &ClimateSampler,
    biome_registry: &BiomeRegistry,
    generation_version: u32,
    world_x: i32,
    world_z: i32,
) -> f64 {
    let mut smoothed = 0.0;
    for dx in -1..=1 {
        for dz in -1..=1 {
            let sample = raw_surface_sample(
                noise_sampler,
                climate_sampler,
                biome_registry,
                generation_version,
                world_x + dx,
                world_z + dz,
            );
            smoothed +=
                sample.height * SURFACE_SMOOTHING_KERNEL[(dx + 1) as usize][(dz + 1) as usize];
        }
    }
    smoothed
}

#[derive(Clone, Copy)]
struct RawSurfaceSample {
    temperature: f64,
    humidity: f64,
    height: f64,
}

/// 地形生成器 - 根据群系参数生成地形
pub struct TerrainGenerator;

impl TerrainGenerator {
    /// 采样任意世界列的平滑基础地表。
    ///
    /// 该函数复用区块生成的噪声、气候、群系混合和 3x3 平滑核，因此同一世界种子、
    /// 生成版本与坐标下的 `ground_height` 必须与 `sample_context` 中对应列完全一致。
    pub fn sample_surface(
        noise_sampler: &NoiseSampler,
        climate_sampler: &ClimateSampler,
        biome_registry: &BiomeRegistry,
        key: BaseGenerationKey,
        world_x: i32,
        world_z: i32,
    ) -> TerrainSurfaceSample {
        debug_assert_eq!(noise_sampler.seed, key.seed);
        debug_assert_eq!(climate_sampler.seed, key.seed);

        let center = raw_surface_sample(
            noise_sampler,
            climate_sampler,
            biome_registry,
            key.generation_version,
            world_x,
            world_z,
        );
        let ground_height = smooth_surface_height(
            noise_sampler,
            climate_sampler,
            biome_registry,
            key.generation_version,
            world_x,
            world_z,
        );

        let ground_height = ground_height.round() as i32;
        let biome_index = select_biome(biome_registry, center.temperature, center.humidity);
        let biome = biome_registry
            .get(biome_index)
            .expect("采样地表时生物群系注册表不能为空");
        // 这里与 `generate_terrain` 的覆盖层分支保持一致：海岸和水下使用 beach，
        // 高于海平面两格后才使用群系的正式地表；水下三层同样由 beach 填充。
        let surface_block = if ground_height <= SEA_LEVEL + 2 {
            biome.beach_block.clone()
        } else {
            biome.surface_block.clone()
        };
        let subsurface_block = if ground_height <= SEA_LEVEL {
            biome.beach_block.clone()
        } else {
            biome.subsurface_block.clone()
        };
        TerrainSurfaceSample {
            ground_height,
            visible_surface_height: ground_height.max(SEA_LEVEL) + 1,
            is_water_surface: ground_height < SEA_LEVEL,
            surface_block,
            subsurface_block,
            temperature: center.temperature,
            humidity: center.humidity,
            biome_index,
        }
    }

    /// 生成区块的气候/群系上下文
    pub fn sample_context(
        noise_sampler: &NoiseSampler,
        climate_sampler: &ClimateSampler,
        biome_registry: &BiomeRegistry,
        key: BaseGenerationKey,
    ) -> ChunkGenContext {
        debug_assert_eq!(noise_sampler.seed, key.seed);
        debug_assert_eq!(climate_sampler.seed, key.seed);
        let chunk_pos = key.chunk_pos;
        let world_start_x = chunk_pos.x * CHUNK_SIZE as i32;
        let world_start_z = chunk_pos.z * CHUNK_SIZE as i32;

        let mut ctx = ChunkGenContext::new(chunk_pos);

        // 扩展一圈以包含邻居边界，使平滑核能真正跨区块采样；远景单点采样复用同一
        // 原始高度函数，但区块生成保留这份批量缓存，避免重复计算相邻列。
        const PADDED: usize = CHUNK_SIZE + 2;
        let mut raw_heights = [[0.0f64; PADDED]; PADDED];
        let mut cached_temperature = [[0.0f64; PADDED]; PADDED];
        let mut cached_humidity = [[0.0f64; PADDED]; PADDED];

        for x in 0..PADDED {
            for z in 0..PADDED {
                let world_x = world_start_x + x as i32 - 1;
                let world_z = world_start_z + z as i32 - 1;
                let sample = raw_surface_sample(
                    noise_sampler,
                    climate_sampler,
                    biome_registry,
                    key.generation_version,
                    world_x,
                    world_z,
                );
                raw_heights[x][z] = sample.height;
                cached_temperature[x][z] = sample.temperature;
                cached_humidity[x][z] = sample.humidity;
            }
        }

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = world_start_x + x as i32;
                let world_z = world_start_z + z as i32;

                let temperature = cached_temperature[x + 1][z + 1];
                let humidity = cached_humidity[x + 1][z + 1];

                // 主要群系（用于 surface_block、tree_density 等）
                let biome_index = select_biome(biome_registry, temperature, humidity);
                let mut final_height = 0.0;
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        let nx = (x + 1) as i32 + dx;
                        let nz = (z + 1) as i32 + dz;
                        final_height += raw_heights[nx as usize][nz as usize]
                            * SURFACE_SMOOTHING_KERNEL[(dx + 1) as usize][(dz + 1) as usize];
                    }
                }

                ctx.columns.push(ColumnContext {
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
        let mut chunk_data = ChunkData {
            voxels: [0u16; CHUNK_VOLUME],
        };
        let world_start_y = ctx.chunk_pos.y * CHUNK_SIZE as i32;

        struct ColCache {
            target_surface_y: i32,
            surface_id: u16,
            subsurface_id: u16,
            beach_id: u16,
        }

        let col_cache: Vec<ColCache> = ctx
            .columns
            .iter()
            .map(|col| {
                let biome = biome_registry.get(col.biome_index).unwrap();
                ColCache {
                    target_surface_y: col.base_height,
                    surface_id: block_ids.resolve_block_id(&biome.surface_block),
                    subsurface_id: block_ids.resolve_block_id(&biome.subsurface_block),
                    beach_id: block_ids.resolve_block_id(&biome.beach_block),
                }
            })
            .collect();

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let cache = &col_cache[x * CHUNK_SIZE + z];
                let target = cache.target_surface_y;
                let surface_id = cache.surface_id;
                let subsurface_id = cache.subsurface_id;
                let beach_id = cache.beach_id;

                for y in 0..CHUNK_SIZE {
                    let world_y = world_start_y + y as i32;

                    let voxel_id = if world_y > target {
                        if world_y <= SEA_LEVEL {
                            block_ids.water
                        } else {
                            block_ids.air
                        }
                    } else if world_y == target {
                        if world_y <= SEA_LEVEL + 2 {
                            beach_id
                        } else {
                            surface_id
                        }
                    } else if world_y > target - 4 {
                        if target <= SEA_LEVEL {
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

#[cfg(test)]
#[path = "../../../../../tests/unit/game/world/generation/terrain/generator.rs"]
mod tests;

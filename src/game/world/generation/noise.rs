use crate::content::block::registry::BlockRegistry;
use crate::engine::constant::world::*;
use crate::content::biome::definition::BiomeRegistry;
use crate::game::world::generation::climate::{ClimateSampler, Season};
use crate::game::world::generation::context::{ChunkGenContext, ColumnContext};
use crate::shared::tag::cache::TagCache;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::HashSet;

/// 地形生成器
pub struct TerrainGenerator {
    perlin: Perlin,
}

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

        const PADDED: usize = CHUNK_SIZE + 2; // 18

        // ── 第一步：稀疏采样气候 → 4×4 ──
        // 同时预计算 blended terrain params，避免后续 exp() 重复计算
        let mut sparse_temp = [[0.0f64; CLIMATE_SPARSE]; CLIMATE_SPARSE];
        let mut sparse_humid = [[0.0f64; CLIMATE_SPARSE]; CLIMATE_SPARSE];
        let mut sparse_blend_base = [[0.0f64; CLIMATE_SPARSE]; CLIMATE_SPARSE];
        let mut sparse_blend_amp = [[0.0f64; CLIMATE_SPARSE]; CLIMATE_SPARSE];
        let mut sparse_blend_rough = [[0.0f64; CLIMATE_SPARSE]; CLIMATE_SPARSE];

        for sx in 0..CLIMATE_SPARSE {
            for sz in 0..CLIMATE_SPARSE {
                // 稀疏索引 → 密集索引 (0..17)
                let di = sx * (PADDED - 1) / (CLIMATE_SPARSE - 1);
                let dj = sz * (PADDED - 1) / (CLIMATE_SPARSE - 1);
                let world_x = world_start_x - 1 + di as i32;
                let world_z = world_start_z - 1 + dj as i32;

                let t = climate_sampler.sample_temperature_with_season(world_x, world_z, season);
                let h = climate_sampler.sample_humidity_with_season(world_x, world_z, season);
                let blended = biome_registry.blend_terrain_params(t, h);

                sparse_temp[sx][sz] = t;
                sparse_humid[sx][sz] = h;
                sparse_blend_base[sx][sz] = blended.base_height;
                sparse_blend_amp[sx][sz] = blended.height_amplitude;
                sparse_blend_rough[sx][sz] = blended.roughness;
            }
        }

        // ── 第二步：稀疏采样地形噪声 → 9×9 ──
        let mut sparse_primary = [[0.0f64; TERRAIN_SPARSE]; TERRAIN_SPARSE];
        let mut sparse_detail = [[0.0f64; TERRAIN_SPARSE]; TERRAIN_SPARSE];
        let mut sparse_rough = [[0.0f64; TERRAIN_SPARSE]; TERRAIN_SPARSE];

        for sx in 0..TERRAIN_SPARSE {
            for sz in 0..TERRAIN_SPARSE {
                let di = sx * (PADDED - 1) / (TERRAIN_SPARSE - 1);
                let dj = sz * (PADDED - 1) / (TERRAIN_SPARSE - 1);
                let world_x = world_start_x - 1 + di as i32;
                let world_z = world_start_z - 1 + dj as i32;

                sparse_primary[sx][sz] = noise_sampler.terrain_primary.get([
                    world_x as f64 * GLOBAL_TERRAIN_SCALE,
                    world_z as f64 * GLOBAL_TERRAIN_SCALE,
                ]);
                sparse_detail[sx][sz] = noise_sampler.terrain_detail.get([
                    world_x as f64 * GLOBAL_DETAIL_SCALE,
                    world_z as f64 * GLOBAL_DETAIL_SCALE,
                ]);
                sparse_rough[sx][sz] = noise_sampler.roughness.get([
                    world_x as f64 * GLOBAL_ROUGHNESS_SCALE,
                    world_z as f64 * GLOBAL_ROUGHNESS_SCALE,
                ]);
            }
        }

        // ── 第三步：全部上采样到 18×18 ──
        let temp = upsample_sparse::<PADDED, CLIMATE_SPARSE>(&sparse_temp);
        let humid = upsample_sparse::<PADDED, CLIMATE_SPARSE>(&sparse_humid);
        let blend_base = upsample_sparse::<PADDED, CLIMATE_SPARSE>(&sparse_blend_base);
        let blend_amp = upsample_sparse::<PADDED, CLIMATE_SPARSE>(&sparse_blend_amp);
        let blend_rough = upsample_sparse::<PADDED, CLIMATE_SPARSE>(&sparse_blend_rough);
        let primary = upsample_sparse::<PADDED, TERRAIN_SPARSE>(&sparse_primary);
        let detail = upsample_sparse::<PADDED, TERRAIN_SPARSE>(&sparse_detail);
        let roughness = upsample_sparse::<PADDED, TERRAIN_SPARSE>(&sparse_rough);

        // ── 第四步：用上采样值直接计算 raw_heights（无噪声调用、无 exp）──
        let mut raw_heights = [[0.0f64; PADDED]; PADDED];

        for x in 0..PADDED {
            for z in 0..PADDED {
                let bbase = blend_base[x][z];
                let bamp = blend_amp[x][z];
                let brough = blend_rough[x][z];
                let p = primary[x][z];
                let d = detail[x][z];
                let r = roughness[x][z];

                let roughness_factor = (r + 1.0) * 0.5 * brough;
                raw_heights[x][z] = bbase + p * bamp + d * bamp * 0.3 * roughness_factor;
            }
        }

        // ── 第五步：3×3 高斯平滑 + 构建 ColumnContext ──
        let kernel = [
            [0.0625, 0.125, 0.0625],
            [0.125, 0.25, 0.125],
            [0.0625, 0.125, 0.0625],
        ];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = world_start_x + x as i32;
                let world_z = world_start_z + z as i32;

                let temperature = temp[x + 1][z + 1];
                let humidity = humid[x + 1][z + 1];

                let biome_index = biome_registry.select_biome(temperature, humidity);

                let mut smoothed = 0.0;
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        let nx = (x + 1) as i32 + dx;
                        let nz = (z + 1) as i32 + dz;
                        smoothed += raw_heights[nx as usize][nz as usize]
                            * kernel[(dx + 1) as usize][(dz + 1) as usize];
                    }
                }

                ctx.columns.push(ColumnContext {
                    world_x,
                    world_z,
                    temperature,
                    humidity,
                    biome_index,
                    base_height: smoothed.round() as i32,
                    roughness: 0.0,
                });
            }
        }

        ctx
    }
}

/// 生成地形主要使用的方块缓存
#[derive(Clone)]
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
    /// 可种树地表方块ID集合
    pub tree_plantable_ids: HashSet<u16>,
    /// 自然方块ID集合
    pub natural_ids: HashSet<u16>,
    /// 可替换方块ID集合
    pub overworld_replaceable_ids: HashSet<u16>,
}

impl GenerationBlockIds {
    /// 游戏在调用生成前，从 Bevy 的中央注册表中一次性把名字翻译成数字 ID
    pub fn from_registry(registry: &BlockRegistry, tag_cache: &TagCache) -> Self {
        Self {
            air: 0,
            grass: registry
                .get_id_by_identifier("century_journey:grass")
                .unwrap_or(0),
            dirt: registry
                .get_id_by_identifier("century_journey:dirt")
                .unwrap_or(0),
            stone: registry
                .get_id_by_identifier("century_journey:stone")
                .unwrap_or(0),
            sand: registry
                .get_id_by_identifier("century_journey:sand")
                .unwrap_or(0),
            water: registry
                .get_id_by_identifier("century_journey:water")
                .unwrap_or(0),
            snow: registry
                .get_id_by_identifier("century_journey:snow")
                .unwrap_or(0),
            leaves: registry
                .get_id_by_identifier("century_journey:leaves")
                .unwrap_or(0),
            wood: registry
                .get_id_by_identifier("century_journey:wood")
                .unwrap_or(0),
            tree_plantable_ids: tag_cache
                .get_block_tag_ids("century_journey:tree_plantable")
                .cloned()
                .unwrap_or_else(|| {
                    // 标签不存在时回退：默认只有草方块可种树
                    let mut set = HashSet::new();
                    if let Some(id) = registry.get_id_by_identifier("century_journey:grass") {
                        set.insert(id);
                    }
                    set
                }),
            natural_ids: tag_cache
                .get_block_tag_ids("century_journey:natural")
                .cloned()
                .unwrap_or_default(),
            overworld_replaceable_ids: tag_cache
                .get_block_tag_ids("century_journey:overworld_replaceable")
                .cloned()
                .unwrap_or_default(),
        }
    }

    /// 查询方块是否可在该地表种树
    pub fn is_tree_plantable(&self, block_id: u16) -> bool {
        self.tree_plantable_ids.contains(&block_id)
    }

    /// 查询方块是否为自然方块
    pub fn is_natural(&self, block_id: u16) -> bool {
        self.natural_ids.contains(&block_id)
    }

    /// 查询方块是否可被主世界替换
    pub fn is_overworld_replaceable(&self, block_id: u16) -> bool {
        self.overworld_replaceable_ids.contains(&block_id)
    }

    /// 从方块标识符解析到ID
    #[deprecated(note = "请使用标签查询替代硬编码标识符匹配")]
    pub fn resolve_block_id(&self, _identifier: &str) -> u16 {
        match _identifier {
            "century_journey:grass" => self.grass,
            "century_journey:dirt" => self.dirt,
            "century_journey:stone" => self.stone,
            "century_journey:sand" => self.sand,
            "century_journey:water" => self.water,
            "century_journey:snow" => self.snow,
            "century_journey:leaves" => self.leaves,
            "century_journey:wood" => self.wood,
            _ => self.grass,
        }
    }
}

/// 缓存方块ID资源，避免每帧重建
#[derive(Resource, Clone)]
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

/// 双线性插值
fn upsample_sparse<const D: usize, const S: usize>(sparse: &[[f64; S]; S]) -> [[f64; D]; D] {
    let mut dense = [[0.0f64; D]; D];
    let s_last = (S - 1) as f64;
    let d_last = (D - 1) as f64;

    for x in 0..D {
        let sx_f = x as f64 * s_last / d_last;
        let sx_lo = sx_f.floor() as usize;
        let sx_hi = (sx_lo + 1).min(S - 1);
        let fx = sx_f - sx_lo as f64;

        for z in 0..D {
            let sz_f = z as f64 * s_last / d_last;
            let sz_lo = sz_f.floor() as usize;
            let sz_hi = (sz_lo + 1).min(S - 1);
            let fz = sz_f - sz_lo as f64;

            // 标准双线性插值
            let v00 = sparse[sx_lo][sz_lo];
            let v10 = sparse[sx_hi][sz_lo];
            let v01 = sparse[sx_lo][sz_hi];
            let v11 = sparse[sx_hi][sz_hi];

            dense[x][z] =
                (v00 * (1.0 - fx) + v10 * fx) * (1.0 - fz) + (v01 * (1.0 - fx) + v11 * fx) * fz;
        }
    }
    dense
}

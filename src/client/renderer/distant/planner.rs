//! 将玩家附近的远景圆环转换为可异步构建的稳定瓦片计划。

use super::config::DistantTerrainConfig;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::math::IVec3;
use std::collections::HashSet;

/// 每个远景瓦片固定使用 16x16 个低分辨率真实方块柱。
pub(super) const DISTANT_TILE_GRID_CELLS: usize = 16;

/// 能唯一标识一个远景真实方块 LOD 瓦片的稳定键。
///
/// 瓦片跨度属于键的一部分，避免玩家修改近景视距后旧任务把不同采样精度的结果
/// 错误提交到同一位置；覆盖位图则保证玩家移动后旧的近景让出边界不会复用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct DistantTerrainTileKey {
    pub(super) lod_level: u8,
    pub(super) origin_chunk_x: i32,
    pub(super) origin_chunk_z: i32,
    pub(super) span_chunks: i32,
    /// 16x16 粗单元是否保留绘制，按行打包成四个 u64。
    pub(super) coverage_mask: [u64; 4],
}

/// 单个远景瓦片的几何范围、采样间距和真实区块裁剪参数。
#[derive(Debug, Clone, Copy)]
pub(super) struct DistantTerrainTileSpec {
    pub(super) key: DistantTerrainTileKey,
    pub(super) sample_step_blocks: i32,
    /// 本瓦片所属 LOD 环的内外区块半径；网格会按真实区块圆盘逐单元裁剪。
    pub(super) inner_radius_chunks: i32,
    pub(super) outer_radius_chunks: i32,
    /// 所有远景环共用的可见范围，用于在不同采样密度之间生成单侧过渡墙。
    pub(super) lod_inner_radius_chunks: i32,
    pub(super) lod_outer_radius_chunks: i32,
    pub(super) player_chunk_x: i32,
    pub(super) player_chunk_z: i32,
}

impl DistantTerrainTileSpec {
    /// 返回瓦片在世界方块坐标中的 X 起点。
    pub(super) fn origin_world_x(self) -> i32 {
        self.key.origin_chunk_x * crate::shared::voxel::CHUNK_SIZE as i32
    }

    /// 返回瓦片在世界方块坐标中的 Z 起点。
    pub(super) fn origin_world_z(self) -> i32 {
        self.key.origin_chunk_z * crate::shared::voxel::CHUNK_SIZE as i32
    }
}

/// 构建从近景网格边界延伸到远处的稳定真实方块 LOD 瓦片计划。
pub(super) fn build_distant_terrain_plan(
    player_chunk: IVec3,
    near_radius_chunks: i32,
    config: &DistantTerrainConfig,
) -> Vec<DistantTerrainTileSpec> {
    let mut plan = Vec::new();
    let mut keys = HashSet::new();
    let far_radius_chunks = config.far_radius_chunks(near_radius_chunks);

    for ring in config.rings(near_radius_chunks) {
        if ring.outer_radius_chunks <= ring.inner_radius_chunks {
            continue;
        }
        let span = ring.tile_span_chunks.max(1);
        let outer = ring.outer_radius_chunks;
        let min_tile_x = (player_chunk.x - outer - span).div_euclid(span);
        let max_tile_x = (player_chunk.x + outer + span).div_euclid(span);
        let min_tile_z = (player_chunk.z - outer - span).div_euclid(span);
        let max_tile_z = (player_chunk.z + outer + span).div_euclid(span);

        for tile_x in min_tile_x..=max_tile_x {
            for tile_z in min_tile_z..=max_tile_z {
                let origin_chunk_x = tile_x * span;
                let origin_chunk_z = tile_z * span;
                if !tile_intersects_ring(
                    player_chunk,
                    origin_chunk_x,
                    origin_chunk_z,
                    span,
                    ring.inner_radius_chunks,
                    ring.outer_radius_chunks,
                ) {
                    continue;
                }

                let mut spec = DistantTerrainTileSpec {
                    key: DistantTerrainTileKey {
                        lod_level: ring.lod_level,
                        origin_chunk_x,
                        origin_chunk_z,
                        span_chunks: span,
                        coverage_mask: [0; 4],
                    },
                    // 步长与瓦片跨度保持 16 个粗单元；4/8 方块步长都能整除区块边长，
                    // 每个粗单元因此准确归属于一个真实区块。
                    sample_step_blocks: span,
                    inner_radius_chunks: ring.inner_radius_chunks,
                    outer_radius_chunks: ring.outer_radius_chunks,
                    lod_inner_radius_chunks: near_radius_chunks,
                    lod_outer_radius_chunks: far_radius_chunks,
                    player_chunk_x: player_chunk.x,
                    player_chunk_z: player_chunk.z,
                };
                spec.key.coverage_mask = coverage_mask(spec);
                let key = spec.key;
                debug_assert!(keys.insert(key));
                plan.push(spec);
            }
        }
    }

    plan.sort_by_key(|spec| {
        let center_x = spec.key.origin_chunk_x + spec.key.span_chunks / 2;
        let center_z = spec.key.origin_chunk_z + spec.key.span_chunks / 2;
        let dx = i64::from(center_x - player_chunk.x);
        let dz = i64::from(center_z - player_chunk.z);
        (dx * dx + dz * dz, spec.key.lod_level, spec.key)
    });
    plan
}

fn tile_intersects_ring(
    player_chunk: IVec3,
    origin_chunk_x: i32,
    origin_chunk_z: i32,
    span_chunks: i32,
    inner_radius_chunks: i32,
    outer_radius_chunks: i32,
) -> bool {
    let min_distance_sq =
        squared_distance_to_tile(player_chunk, origin_chunk_x, origin_chunk_z, span_chunks);
    let outer_sq = squared_radius(outer_radius_chunks);
    if min_distance_sq > outer_sq {
        return false;
    }

    farthest_corner_distance_sq(player_chunk, origin_chunk_x, origin_chunk_z, span_chunks)
        >= squared_radius(inner_radius_chunks)
}

fn squared_distance_to_tile(
    player_chunk: IVec3,
    origin_chunk_x: i32,
    origin_chunk_z: i32,
    span_chunks: i32,
) -> i64 {
    let max_chunk_x = origin_chunk_x + span_chunks.saturating_sub(1);
    let max_chunk_z = origin_chunk_z + span_chunks.saturating_sub(1);
    let dx = axis_distance_to_range(player_chunk.x, origin_chunk_x, max_chunk_x);
    let dz = axis_distance_to_range(player_chunk.z, origin_chunk_z, max_chunk_z);
    squared_components(dx, dz)
}

fn farthest_corner_distance_sq(
    player_chunk: IVec3,
    origin_chunk_x: i32,
    origin_chunk_z: i32,
    span_chunks: i32,
) -> i64 {
    let max_chunk_x = origin_chunk_x + span_chunks.saturating_sub(1);
    let max_chunk_z = origin_chunk_z + span_chunks.saturating_sub(1);
    [
        squared_components(
            origin_chunk_x - player_chunk.x,
            origin_chunk_z - player_chunk.z,
        ),
        squared_components(
            origin_chunk_x - player_chunk.x,
            max_chunk_z - player_chunk.z,
        ),
        squared_components(
            max_chunk_x - player_chunk.x,
            origin_chunk_z - player_chunk.z,
        ),
        squared_components(max_chunk_x - player_chunk.x, max_chunk_z - player_chunk.z),
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
}

fn axis_distance_to_range(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min - value
    } else if value > max {
        value - max
    } else {
        0
    }
}

fn squared_radius(radius: i32) -> i64 {
    let radius = i64::from(radius.max(0));
    radius * radius
}

fn squared_components(x: i32, z: i32) -> i64 {
    let x = i64::from(x);
    let z = i64::from(z);
    x * x + z * z
}

/// 判断一个粗单元是否需要由远景 LOD 绘制。
///
/// 单元覆盖的所有真实区块都在近景圆盘内时整格让出；只要覆盖外环中的任意区块，
/// 就保留该格。这个边界规则与网格采样共用，避免瓦片重建时出现不同的裁剪结果。
pub(super) fn cell_in_lod_ring(spec: DistantTerrainTileSpec, cell_x: isize, cell_z: isize) -> bool {
    let step = spec.sample_step_blocks.max(1);
    let min_x = spec.origin_world_x() + cell_x as i32 * step;
    let min_z = spec.origin_world_z() + cell_z as i32 * step;
    let inner = i64::from(spec.inner_radius_chunks.max(0));
    let outer = i64::from(spec.outer_radius_chunks.max(0));
    let max_x = min_x + step - 1;
    let max_z = min_z + step - 1;
    let first_chunk_x = min_x.div_euclid(CHUNK_SIZE as i32);
    let last_chunk_x = max_x.div_euclid(CHUNK_SIZE as i32);
    let first_chunk_z = min_z.div_euclid(CHUNK_SIZE as i32);
    let last_chunk_z = max_z.div_euclid(CHUNK_SIZE as i32);

    let mut touches_outer_ring = false;
    for chunk_x in first_chunk_x..=last_chunk_x {
        for chunk_z in first_chunk_z..=last_chunk_z {
            let dx = i64::from(chunk_x - spec.player_chunk_x);
            let dz = i64::from(chunk_z - spec.player_chunk_z);
            let distance_sq = dx * dx + dz * dz;
            if distance_sq <= inner * inner {
                return false;
            }
            touches_outer_ring |= distance_sq <= outer * outer;
        }
    }
    touches_outer_ring
}

/// 判断粗单元是否在整个远景可见范围内，供两个不同 LOD 环的接缝生成过渡墙。
pub(super) fn cell_in_any_lod_ring(
    spec: DistantTerrainTileSpec,
    cell_x: isize,
    cell_z: isize,
) -> bool {
    let step = spec.sample_step_blocks.max(1);
    let min_x = spec.origin_world_x() + cell_x as i32 * step;
    let min_z = spec.origin_world_z() + cell_z as i32 * step;
    let inner = i64::from(spec.lod_inner_radius_chunks.max(0));
    let outer = i64::from(spec.lod_outer_radius_chunks.max(0));
    let max_x = min_x + step - 1;
    let max_z = min_z + step - 1;
    let first_chunk_x = min_x.div_euclid(CHUNK_SIZE as i32);
    let last_chunk_x = max_x.div_euclid(CHUNK_SIZE as i32);
    let first_chunk_z = min_z.div_euclid(CHUNK_SIZE as i32);
    let last_chunk_z = max_z.div_euclid(CHUNK_SIZE as i32);

    let mut touches_outer_range = false;
    for chunk_x in first_chunk_x..=last_chunk_x {
        for chunk_z in first_chunk_z..=last_chunk_z {
            let dx = i64::from(chunk_x - spec.player_chunk_x);
            let dz = i64::from(chunk_z - spec.player_chunk_z);
            let distance_sq = dx * dx + dz * dz;
            if distance_sq <= inner * inner {
                return false;
            }
            touches_outer_range |= distance_sq <= outer * outer;
        }
    }
    touches_outer_range
}

/// 判断粗单元是否触及近景真实区块圆盘，用于只填补 LOD0 的高度差。
pub(super) fn cell_touches_near_region(
    spec: DistantTerrainTileSpec,
    cell_x: isize,
    cell_z: isize,
) -> bool {
    let step = spec.sample_step_blocks.max(1);
    let min_x = spec.origin_world_x() + cell_x as i32 * step;
    let min_z = spec.origin_world_z() + cell_z as i32 * step;
    let radius = i64::from(spec.lod_inner_radius_chunks.max(0));
    let max_x = min_x + step - 1;
    let max_z = min_z + step - 1;
    let first_chunk_x = min_x.div_euclid(CHUNK_SIZE as i32);
    let last_chunk_x = max_x.div_euclid(CHUNK_SIZE as i32);
    let first_chunk_z = min_z.div_euclid(CHUNK_SIZE as i32);
    let last_chunk_z = max_z.div_euclid(CHUNK_SIZE as i32);

    for chunk_x in first_chunk_x..=last_chunk_x {
        for chunk_z in first_chunk_z..=last_chunk_z {
            let dx = i64::from(chunk_x - spec.player_chunk_x);
            let dz = i64::from(chunk_z - spec.player_chunk_z);
            if dx * dx + dz * dz <= radius * radius {
                return true;
            }
        }
    }
    false
}

fn coverage_mask(spec: DistantTerrainTileSpec) -> [u64; 4] {
    let mut mask = [0_u64; 4];
    for cell_z in 0..DISTANT_TILE_GRID_CELLS {
        for cell_x in 0..DISTANT_TILE_GRID_CELLS {
            if !cell_in_lod_ring(spec, cell_x as isize, cell_z as isize) {
                continue;
            }
            let bit = cell_z * DISTANT_TILE_GRID_CELLS + cell_x;
            mask[bit / 64] |= 1_u64 << (bit % 64);
        }
    }
    mask
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/distant/planner.rs"]
mod tests;

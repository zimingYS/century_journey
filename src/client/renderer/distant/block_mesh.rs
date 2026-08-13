//! 把地表采样转换为低分辨率的真实方块柱网格。
//!
//! 远景单元仍然使用地表、次表、石头和水方块的真实运行时 ID，只是把多个连续列
//! 合并成一个粗柱。这样 LOD 与近景区块共享生成数据和图集，而不是另画一套地形。

use super::planner::{
    DISTANT_TILE_GRID_CELLS, DistantTerrainTileSpec, cell_in_any_lod_ring, cell_in_lod_ring,
    cell_touches_near_region,
};
use crate::client::renderer::constants::WATER_SURFACE_INSET;
use crate::client::renderer::world::{BlockInfoSnapshot, MeshBufferData};
use crate::game::world::generation::pipeline::{
    ResolvedTerrainSurfaceSample, TerrainSurfaceSampler,
};
use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::*;

/// 远景柱体的底部；只保留可见侧壁所需的地质剖面。
const DISTANT_COLUMN_BOTTOM_Y: i32 = -64;
/// 远景没有独立 ChunkLight 快照时使用的中性光色。
const DISTANT_LIGHT_COLOR: [f32; 4] = [0.92, 0.92, 0.92, 1.0];

/// 后台任务提交给主线程的两个材质通道网格数据。
pub(super) struct DistantTerrainBlockMeshData {
    /// 不透明地表、次表和石头侧壁。
    pub(super) opaque: MeshBufferData,
    /// 水面与水体侧壁。
    pub(super) water: MeshBufferData,
}

/// 生成一块与真实区块数据同源的低分辨率方块柱网格。
///
/// 采样点位于粗单元中心；真实近景区块圆盘内的单元整格裁掉，避免 LOD 几何穿入
/// 近景网格。相邻单元只在高度较高的一侧生成侧壁，且每个侧壁按真实地层分层贴图。
pub(super) fn build_distant_block_mesh(
    sampler: &TerrainSurfaceSampler,
    block_info: &BlockInfoSnapshot,
    spec: DistantTerrainTileSpec,
) -> DistantTerrainBlockMeshData {
    let grid = DISTANT_TILE_GRID_CELLS;
    let padded = grid + 2;
    let step = spec.sample_step_blocks.max(1);
    let origin_x = spec.origin_world_x();
    let origin_z = spec.origin_world_z();
    let mut samples = Vec::with_capacity(padded * padded);

    // 邻域采样让相邻瓦片在边界比较同一组列，避免侧壁出现缝隙。
    for z in 0..padded {
        for x in 0..padded {
            let world_x = origin_x + (x as i32 - 1) * step + step / 2;
            let world_z = origin_z + (z as i32 - 1) * step + step / 2;
            samples.push(sampler.sample_surface(world_x, world_z));
        }
    }

    let mut opaque = MeshBufferData::with_capacity(grid * grid * 8);
    let mut water = MeshBufferData::with_capacity(grid * grid * 3);

    for z in 0..grid {
        for x in 0..grid {
            let index = (z + 1) * padded + x + 1;
            let current = samples[index];
            if !cell_in_lod_ring(spec, x as isize, z as isize) {
                continue;
            }

            let x0 = (x as i32 * step) as f32;
            let x1 = ((x as i32 + 1) * step) as f32;
            let z0 = (z as i32 * step) as f32;
            let z1 = ((z as i32 + 1) * step) as f32;
            let ground_top = current.ground_height + 1;
            let current_visible_top = visible_top(current);

            if current.is_water_surface {
                append_top_face(
                    &mut water,
                    current.water_block,
                    x0,
                    x1,
                    current_visible_top,
                    z0,
                    z1,
                    block_info,
                    true,
                );
            } else {
                append_top_face(
                    &mut opaque,
                    current.surface_block,
                    x0,
                    x1,
                    ground_top as f32,
                    z0,
                    z1,
                    block_info,
                    false,
                );
            }

            for (face_idx, dx, dz, edge) in [
                (2usize, -1isize, 0isize, Edge::West),
                (3, 1, 0, Edge::East),
                (5, 0, -1, Edge::North),
                (4, 0, 1, Edge::South),
            ] {
                // 近景圆盘与不同采样密度的环带只生成高度差裙边；远景外缘交给雾效，
                // 同一 LOD 环内部则只由较高的一侧生成侧壁，避免共面网格闪烁。
                let neighbor_cell_x = x as isize + dx;
                let neighbor_cell_z = z as isize + dz;
                let neighbor_in_same_ring =
                    cell_in_lod_ring(spec, neighbor_cell_x, neighbor_cell_z);
                let neighbor_in_any_ring =
                    cell_in_any_lod_ring(spec, neighbor_cell_x, neighbor_cell_z);
                let transition_wall = if spec.key.lod_level == 0 {
                    !neighbor_in_same_ring
                        && cell_touches_near_region(spec, neighbor_cell_x, neighbor_cell_z)
                } else {
                    !neighbor_in_same_ring && neighbor_in_any_ring
                };
                if !neighbor_in_same_ring && !transition_wall {
                    continue;
                }
                let neighbor_index = (index as isize + dz * padded as isize + dx) as usize;
                let neighbor = samples[neighbor_index];
                if current_visible_top <= visible_top(neighbor) {
                    continue;
                }
                let wall_top = if transition_wall {
                    current_visible_top.ceil() as i32
                } else {
                    ground_top
                };
                let wall_bottom = visible_top(neighbor).ceil() as i32;
                let (ex0, ex1, ez0, ez1) = edge.bounds(x0, x1, z0, z1);
                if current.is_water_surface {
                    append_land_wall(
                        &mut opaque,
                        current,
                        face_idx,
                        ex0,
                        ex1,
                        ez0,
                        ez1,
                        wall_bottom,
                        wall_top,
                        block_info,
                    );
                    append_vertical_face(
                        &mut water,
                        current.water_block,
                        face_idx,
                        ex0,
                        ex1,
                        ez0,
                        ez1,
                        ground_top as f32,
                        visible_top(current),
                        block_info,
                        true,
                    );
                } else {
                    append_land_wall(
                        &mut opaque,
                        current,
                        face_idx,
                        ex0,
                        ex1,
                        ez0,
                        ez1,
                        wall_bottom,
                        wall_top,
                        block_info,
                    );
                }
            }
        }
    }

    DistantTerrainBlockMeshData { opaque, water }
}

#[derive(Clone, Copy)]
enum Edge {
    West,
    East,
    North,
    South,
}

impl Edge {
    fn bounds(self, x0: f32, x1: f32, z0: f32, z1: f32) -> (f32, f32, f32, f32) {
        match self {
            Self::West => (x0, x0, z0, z1),
            Self::East => (x1, x1, z0, z1),
            Self::North => (x0, x1, z0, z0),
            Self::South => (x0, x1, z1, z1),
        }
    }
}

fn visible_top(sample: ResolvedTerrainSurfaceSample) -> f32 {
    if sample.is_water_surface {
        sample.visible_surface_height as f32 - WATER_SURFACE_INSET
    } else {
        (sample.ground_height + 1) as f32
    }
}

// 单个墙面同时携带方向、范围和三层地质材质；拆成多个对象会让后台热路径产生大量临时值。
// 顶面需要同时携带粗单元范围、纹理层和水面 UV 语义，保持热路径为标量参数。
#[allow(clippy::too_many_arguments)]
fn append_land_wall(
    buffer: &mut MeshBufferData,
    sample: ResolvedTerrainSurfaceSample,
    face_idx: usize,
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    bottom: i32,
    top: i32,
    block_info: &BlockInfoSnapshot,
) {
    let bottom = bottom.max(DISTANT_COLUMN_BOTTOM_Y).min(top);
    let surface_bottom = (top - 1).max(bottom);
    let subsurface_bottom = (top - 1 - 3).max(bottom);
    if bottom < subsurface_bottom {
        append_vertical_face(
            buffer,
            sample.stone_block,
            face_idx,
            x0,
            x1,
            z0,
            z1,
            bottom as f32,
            subsurface_bottom as f32,
            block_info,
            false,
        );
    }
    if subsurface_bottom < surface_bottom {
        append_vertical_face(
            buffer,
            sample.subsurface_block,
            face_idx,
            x0,
            x1,
            z0,
            z1,
            subsurface_bottom as f32,
            surface_bottom as f32,
            block_info,
            false,
        );
    }
    if surface_bottom < top {
        append_vertical_face(
            buffer,
            sample.surface_block,
            face_idx,
            x0,
            x1,
            z0,
            z1,
            surface_bottom as f32,
            top as f32,
            block_info,
            false,
        );
    }
}

// 顶面需要同时携带粗单元范围、纹理层和水面 UV 语义，保持热路径为标量参数。
#[allow(clippy::too_many_arguments)]
fn append_top_face(
    buffer: &mut MeshBufferData,
    block_id: u16,
    x0: f32,
    x1: f32,
    y: f32,
    z0: f32,
    z1: f32,
    block_info: &BlockInfoSnapshot,
    water_uv: bool,
) {
    let texture_layer = block_info.get_texture_layer(block_id, 0);
    let total_layers = block_info.total_layers.max(1) as f32;
    let width = (x1 - x0) / CHUNK_SIZE as f32;
    let depth = (z1 - z0) / CHUNK_SIZE as f32;
    let (u0, u1, v0, v1) = if water_uv {
        (0.0, x1 - x0, 0.0, z1 - z0)
    } else {
        (
            0.0,
            width,
            texture_layer as f32 / total_layers,
            (texture_layer as f32 + depth) / total_layers,
        )
    };
    let vertices = [[x1, y, z0], [x0, y, z0], [x0, y, z1], [x1, y, z1]];
    buffer.append_face(
        &vertices,
        Vec3::Y,
        &[[u1, v0], [u0, v0], [u0, v1], [u1, v1]],
        DISTANT_LIGHT_COLOR,
        [0.0, 0.0],
    );
}

// 侧面需要同时携带方向、垂直分层范围和材质通道，拆分会增加后台网格临时对象。
#[allow(clippy::too_many_arguments)]
fn append_vertical_face(
    buffer: &mut MeshBufferData,
    block_id: u16,
    face_idx: usize,
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    y0: f32,
    y1: f32,
    block_info: &BlockInfoSnapshot,
    water_uv: bool,
) {
    if y1 <= y0 {
        return;
    }
    let texture_layer = block_info.get_texture_layer(block_id, face_idx);
    let total_layers = block_info.total_layers.max(1) as f32;
    let horizontal = (x1 - x0).abs().max((z1 - z0).abs()) / CHUNK_SIZE as f32;
    let vertical = (y1 - y0).abs() / CHUNK_SIZE as f32;
    let (u0, u1, v0, v1) = if water_uv {
        (
            0.0,
            horizontal * CHUNK_SIZE as f32,
            0.0,
            vertical * CHUNK_SIZE as f32,
        )
    } else {
        (
            0.0,
            horizontal,
            texture_layer as f32 / total_layers,
            (texture_layer as f32 + vertical) / total_layers,
        )
    };
    let (vertices, normal) = match face_idx {
        2 => (
            [[x0, y1, z1], [x0, y1, z0], [x0, y0, z0], [x0, y0, z1]],
            -Vec3::X,
        ),
        3 => (
            [[x1, y1, z0], [x1, y1, z1], [x1, y0, z1], [x1, y0, z0]],
            Vec3::X,
        ),
        4 => (
            [[x1, y1, z1], [x0, y1, z1], [x0, y0, z1], [x1, y0, z1]],
            Vec3::Z,
        ),
        5 => (
            [[x0, y1, z0], [x1, y1, z0], [x1, y0, z0], [x0, y0, z0]],
            -Vec3::Z,
        ),
        _ => return,
    };
    buffer.append_face(
        &vertices,
        normal,
        &[[u1, v0], [u0, v0], [u0, v1], [u1, v1]],
        [0.78, 0.78, 0.78, 1.0],
        [0.0, 0.0],
    );
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/distant/block_mesh.rs"]
mod tests;

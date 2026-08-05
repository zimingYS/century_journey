//! 使用贪心合并生成区块表面，并单独处理透明和交叉平面方块。

use crate::content::block::definition::RenderMode;
use crate::content::block::model::generate_cross_vertices;
use crate::game::world::chunk::ChunkData;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::*;
use std::sync::Arc;

/// 面掩码中表示当前位置没有可生成表面的哨兵值。
const FACE_NONE: u32 = u32::MAX;
/// 水面相对完整方块顶面的下沉距离。
const WATER_SURFACE_INSET: f32 = 0.12;

use super::{BlockInfoSnapshot, DIRECTIONS, MeshBufferData, MeshBuildInput};

/// 构建贪心网格
pub fn build_greedy_mesh(input: MeshBuildInput) -> super::channel::MeshBuildResult {
    let MeshBuildInput {
        chunk_pos,
        current_data,
        neighbors,
        block_info,
    } = input;

    let mut opaque_buf = MeshBufferData::new();
    let mut cutout_buf = MeshBufferData::new();
    let mut water_buf = MeshBufferData::new();

    let cs = CHUNK_SIZE;
    let mut mask = [[0u32; 16]; 16];

    for (face_idx, (dir, _)) in DIRECTIONS.iter().copied().enumerate() {
        let (depth_axis, mx_axis, my_axis) = match face_idx {
            0 | 1 => (1, 0, 2), // Top/Bottom: depth=Y
            2 | 3 => (0, 2, 1), // Left/Right: depth=X
            4 | 5 => (2, 0, 1), // Front/Back: depth=Z
            _ => unreachable!(),
        };

        for depth in 0..cs {
            // 构建面遮罩
            for (my, row) in mask.iter_mut().enumerate().take(cs) {
                for (mx, face_key) in row.iter_mut().enumerate().take(cs) {
                    let (x, y, z) = decode_mask_to_xyz(mx, my, depth, depth_axis, mx_axis, my_axis);

                    let voxel_id = current_data.get_voxel(x, y, z);
                    if voxel_id == 0 {
                        *face_key = FACE_NONE;
                        continue;
                    }

                    if block_info.is_cross_model(voxel_id) {
                        *face_key = FACE_NONE;
                        continue;
                    }

                    let current_is_water = voxel_id == block_info.water_id;

                    let neighbor_pos = IVec3::new(x as i32, y as i32, z as i32) + dir;
                    let is_visible = match get_neighbor_voxel_id_snapshot(
                        neighbor_pos,
                        &current_data,
                        &neighbors,
                        dir,
                    ) {
                        Some(nbr_id) => {
                            is_face_visible_snapshot(current_is_water, nbr_id, &block_info)
                        }
                        None => !current_is_water,
                    };

                    if !is_visible {
                        *face_key = FACE_NONE;
                        continue;
                    }

                    let texture_layer = block_info.get_texture_layer(voxel_id, face_idx);

                    let idx = voxel_id as usize;
                    let buffer_idx = if current_is_water {
                        2u8
                    } else {
                        if idx < block_info.render_modes.len()
                            && block_info.render_modes[idx] == RenderMode::Cutout
                        {
                            1
                        } else {
                            0
                        }
                    };

                    *face_key = texture_layer * 4 + buffer_idx as u32 + 1;
                }
            }

            greedy_merge_pass(
                face_idx,
                depth,
                depth_axis,
                mx_axis,
                my_axis,
                &mut mask,
                &block_info,
                &mut opaque_buf,
                &mut cutout_buf,
                &mut water_buf,
            );
        }
    }

    append_cross_models(chunk_pos, &current_data, &block_info, &mut cutout_buf);

    super::channel::MeshBuildResult {
        chunk_pos,
        opaque: opaque_buf,
        cutout: cutout_buf,
        water: water_buf,
    }
}

/// 将 mask 坐标解码为区块体素坐标
#[inline]
fn decode_mask_to_xyz(
    mx: usize,
    my: usize,
    depth: usize,
    depth_axis: usize,
    mx_axis: usize,
    my_axis: usize,
) -> (usize, usize, usize) {
    let mut coords = [0usize; 3];
    coords[depth_axis] = depth;
    coords[mx_axis] = mx;
    coords[my_axis] = my;
    (coords[0], coords[1], coords[2])
}

/// 对一个面切片执行贪心合并。
// 轴映射、掩码和三个材质通道构成一次局部算法上下文，均为显式借用。
#[allow(clippy::too_many_arguments)]
fn greedy_merge_pass(
    face_idx: usize,
    depth: usize,
    depth_axis: usize,
    mx_axis: usize,
    my_axis: usize,
    mask: &mut [[u32; 16]; 16],
    block_info: &BlockInfoSnapshot,
    opaque_buf: &mut MeshBufferData,
    cutout_buf: &mut MeshBufferData,
    water_buf: &mut MeshBufferData,
) {
    let cs = CHUNK_SIZE;

    for my in 0..cs {
        let mut mx = 0;
        while mx < cs {
            let face_key = mask[my][mx];
            if face_key == FACE_NONE {
                mx += 1;
                continue;
            }

            let decoded = face_key - 1;
            let texture_layer = decoded / 4;
            let buffer_idx = (decoded % 4) as u8;

            // 向右扩展宽度
            let mut width = 1;
            while mx + width < cs && mask[my][mx + width] == face_key {
                width += 1;
            }

            // 向下扩展高度
            let mut height = 1;
            'h_loop: while my + height < cs {
                for dx in 0..width {
                    if mask[my + height][mx + dx] != face_key {
                        break 'h_loop;
                    }
                }
                height += 1;
            }

            let (mut positions, uvs) = get_merged_face_data(
                mx,
                my,
                depth,
                width,
                height,
                face_idx,
                depth_axis,
                mx_axis,
                my_axis,
                texture_layer,
                block_info.total_layers,
                buffer_idx == 2,
            );
            if buffer_idx == 2 {
                inset_water_surface(&mut positions, face_idx);
            }
            let (_, normal) = DIRECTIONS[face_idx];

            let buf = match buffer_idx {
                2 => &mut *water_buf,
                1 => &mut *cutout_buf,
                _ => &mut *opaque_buf,
            };
            buf.append_face(&positions, normal, &uvs);

            for dy in 0..height {
                for dx in 0..width {
                    mask[my + dy][mx + dx] = FACE_NONE;
                }
            }

            mx += width;
        }
    }
}

fn inset_water_surface(positions: &mut [[f32; 3]; 4], face_idx: usize) {
    match face_idx {
        0 => {
            for position in positions {
                position[1] -= WATER_SURFACE_INSET;
            }
        }
        2..=5 => {
            let top = positions
                .iter()
                .map(|position| position[1])
                .fold(f32::NEG_INFINITY, f32::max);
            for position in positions {
                if (position[1] - top).abs() <= f32::EPSILON {
                    position[1] -= WATER_SURFACE_INSET;
                }
            }
        }
        _ => {}
    }
}

/// 生成合并面的顶点坐标和 UV 坐标。
// 几何参数全部为小型标量，封装成临时对象不会减少调用方的认知负担。
#[allow(clippy::too_many_arguments)]
fn get_merged_face_data(
    mx: usize,
    my: usize,
    depth: usize,
    width: usize,
    height: usize,
    face_idx: usize,
    depth_axis: usize,
    mx_axis: usize,
    my_axis: usize,
    texture_layer: u32,
    total_layers: u32,
    water_uv: bool,
) -> ([[f32; 3]; 4], [[f32; 2]; 4]) {
    let cs = CHUNK_SIZE as f32;
    let nt = total_layers as f32;
    let w = width as f32;
    let h = height as f32;

    let mut base = [0.0f32; 3];
    base[depth_axis] = depth as f32;
    base[mx_axis] = mx as f32;
    base[my_axis] = my as f32;

    let mut extent = [0.0f32; 3];
    extent[mx_axis] = w;
    extent[my_axis] = h;

    // 水面使用独立的可平铺 repeat 纹理：UV 为每方块一张完整纹理，
    // 由客户端水面动画系统平移 uv_transform 实现流动。
    let (u0, u1, v0, v1) = if water_uv {
        (0.0, w, 0.0, h)
    } else {
        // 普通图集 UV（平铺式）
        (
            0.0,
            w / cs,
            (texture_layer as f32 * cs) / (nt * cs),
            (texture_layer as f32 * cs + h) / (nt * cs),
        )
    };

    match face_idx {
        0 => {
            // 顶面（Y 正方向）
            let positions = [
                [base[0] + extent[0], base[1] + 1.0, base[2]],
                [base[0], base[1] + 1.0, base[2]],
                [base[0], base[1] + 1.0, base[2] + extent[2]],
                [base[0] + extent[0], base[1] + 1.0, base[2] + extent[2]],
            ];
            let uvs = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            (positions, uvs)
        }
        1 => {
            // 底面（Y 负方向）
            let positions = [
                [base[0], base[1], base[2]],
                [base[0] + extent[0], base[1], base[2]],
                [base[0] + extent[0], base[1], base[2] + extent[2]],
                [base[0], base[1], base[2] + extent[2]],
            ];
            let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
            (positions, uvs)
        }
        2 => {
            // 左面（X 负方向）
            let positions = [
                [base[0], base[1] + extent[1], base[2] + extent[2]],
                [base[0], base[1] + extent[1], base[2]],
                [base[0], base[1], base[2]],
                [base[0], base[1], base[2] + extent[2]],
            ];
            let uvs = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            (positions, uvs)
        }
        3 => {
            // 右面（X 正方向）
            let positions = [
                [base[0] + 1.0, base[1] + extent[1], base[2]],
                [base[0] + 1.0, base[1] + extent[1], base[2] + extent[2]],
                [base[0] + 1.0, base[1], base[2] + extent[2]],
                [base[0] + 1.0, base[1], base[2]],
            ];
            let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
            (positions, uvs)
        }
        4 => {
            // 前面（Z 正方向）
            let positions = [
                [base[0] + extent[0], base[1] + extent[1], base[2] + 1.0],
                [base[0], base[1] + extent[1], base[2] + 1.0],
                [base[0], base[1], base[2] + 1.0],
                [base[0] + extent[0], base[1], base[2] + 1.0],
            ];
            let uvs = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            (positions, uvs)
        }
        5 => {
            // 后面（Z 负方向）
            let positions = [
                [base[0], base[1] + extent[1], base[2]],
                [base[0] + extent[0], base[1] + extent[1], base[2]],
                [base[0] + extent[0], base[1], base[2]],
                [base[0], base[1], base[2]],
            ];
            let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
            (positions, uvs)
        }
        _ => unreachable!(),
    }
}

/// 邻居查询
fn get_neighbor_voxel_id_snapshot(
    neighbor_local_pos: IVec3,
    current_chunk_data: &ChunkData,
    neighbors: &[Option<Arc<ChunkData>>; 6],
    dir: IVec3,
) -> Option<u16> {
    if let Some(nbr_id) = current_chunk_data.get_voxel_safe(
        neighbor_local_pos.x,
        neighbor_local_pos.y,
        neighbor_local_pos.z,
    ) {
        return Some(nbr_id);
    }
    let face_idx = DIRECTIONS.iter().position(|(d, _)| *d == dir)?;
    let neighbor_chunk = neighbors[face_idx].as_deref()?;
    let nx = neighbor_local_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ny = neighbor_local_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let nz = neighbor_local_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;
    Some(neighbor_chunk.get_voxel(nx, ny, nz))
}

/// 判断某个面是否需要渲染
fn is_face_visible_snapshot(
    current_is_water: bool,
    neighbor_voxel_id: u16,
    block_info: &BlockInfoSnapshot,
) -> bool {
    if neighbor_voxel_id == 0 {
        return true;
    }
    if current_is_water {
        return false;
    }
    let nbr_is_solid = block_info
        .is_solid
        .get(neighbor_voxel_id as usize)
        .copied()
        .unwrap_or(true);
    !nbr_is_solid || neighbor_voxel_id == block_info.water_id
}

/// 为区块内的十字模型方块生存独立的双面交叉平面
fn append_cross_models(
    chunk_pos: IVec3,
    current_data: &ChunkData,
    block_info: &BlockInfoSnapshot,
    cutout_buf: &mut MeshBufferData,
) {
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // 不是十字模型直接返回
                let voxel_id = current_data.get_voxel(x, y, z);
                if voxel_id == 0 || !block_info.is_cross_model(voxel_id) {
                    continue;
                }

                let texture_layer = block_info.get_texture_layer(voxel_id, 0);
                let uvs = get_single_block_uvs(texture_layer, block_info.total_layers);
                let world_position =
                    chunk_pos * CHUNK_SIZE as i32 + IVec3::new(x as i32, y as i32, z as i32);
                let rotation = block_info
                    .uses_random_model_rotation(voxel_id)
                    .then(|| cross_rotation_from_world_position(world_position));

                for mut vertices in generate_cross_vertices(x as f32, y as f32, z as f32) {
                    if let Some(rotation) = rotation {
                        let block_center =
                            Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                        rotate_vertices_around_block_center(&mut vertices, block_center, rotation);
                    }

                    let normal = calculate_face_normal(&vertices);
                    cutout_buf.append_face(&vertices, normal, &uvs);
                }
            }
        }
    }
}

/// 计算四边形正面的法线，保证十字模型接受正确光照。
fn calculate_face_normal(vertices: &[[f32; 3]; 4]) -> Vec3 {
    let first = Vec3::from(vertices[0]);
    let second = Vec3::from(vertices[1]);
    let third = Vec3::from(vertices[2]);

    (second - first).cross(third - first).normalize()
}

/// 返回一个方块纹理在纵向图集中的 UV 坐标。
fn get_single_block_uvs(texture_layer: u32, total_layers: u32) -> [[f32; 2]; 4] {
    let chunk_size = CHUNK_SIZE as f32;
    let layer_count = total_layers.max(1) as f32;

    let u0 = 0.0;
    let u1 = 1.0 / chunk_size;
    let v0 = texture_layer as f32 / layer_count;
    let v1 = (texture_layer as f32 + 1.0 / chunk_size) / layer_count;

    [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
}

/// 根据世界坐标生成稳定的十字模型旋转。
fn cross_rotation_from_world_position(position: IVec3) -> Quat {
    let hash = position.x.wrapping_mul(73_856_093)
        ^ position.y.wrapping_mul(19_349_663)
        ^ position.z.wrapping_mul(83_492_791);
    let step = hash.rem_euclid(4) as f32;

    Quat::from_rotation_y(step * std::f32::consts::FRAC_PI_8)
}

/// 围绕当前方块的局部中心旋转模型顶点。
fn rotate_vertices_around_block_center(vertices: &mut [[f32; 3]; 4], center: Vec3, rotation: Quat) {
    for vertex in vertices {
        let position = Vec3::from(*vertex);
        *vertex = (rotation * (position - center) + center).into();
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/world/greedy_mesh.rs"]
mod tests;

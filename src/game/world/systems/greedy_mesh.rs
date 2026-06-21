use std::sync::Arc;
use bevy::prelude::*;
use crate::engine::constant::world::*;
use crate::content::block::properties::RenderMode;
use crate::game::world::chunk::ChunkData;
use crate::game::world::systems::{BlockInfoSnapshot, MeshBufferData, MeshBuildInput, DIRECTIONS};

/// 构建贪心网格
pub fn build_greedy_mesh(input: MeshBuildInput) -> super::channel::MeshBuildResult {
    let MeshBuildInput { chunk_pos, current_data, neighbors, block_info } = input;

    let mut opaque_buf = MeshBufferData::new();
    let mut cutout_buf = MeshBufferData::new();
    let mut water_buf = MeshBufferData::new();

    let cs = CHUNK_SIZE;
    let mut mask = [[0u32; 16]; 16];

    for face_idx in 0..6 {
        let (dir, _) = DIRECTIONS[face_idx];

        let (depth_axis, mx_axis, my_axis) = match face_idx {
            0 | 1 => (1, 0, 2), // Top/Bottom: depth=Y
            2 | 3 => (0, 2, 1), // Left/Right: depth=X
            4 | 5 => (2, 0, 1), // Front/Back: depth=Z
            _ => unreachable!(),
        };

        for depth in 0..cs {
            // 构建面遮罩
            for my in 0..cs {
                for mx in 0..cs {
                    let (x, y, z) = decode_mask_to_xyz(mx, my, depth, depth_axis, mx_axis, my_axis);

                    let voxel_id = current_data.get_voxel(x, y, z);
                    if voxel_id == 0 {
                        mask[my][mx] = FACE_NONE;
                        continue;
                    }

                    let current_is_water = voxel_id == block_info.water_id;

                    let neighbor_pos = IVec3::new(x as i32, y as i32, z as i32) + dir;
                    let is_visible = match get_neighbor_voxel_id_snapshot(
                        neighbor_pos, &current_data, &neighbors, dir,
                    ) {
                        Some(nbr_id) => is_face_visible_snapshot(current_is_water, nbr_id, &block_info),
                        None => !current_is_water,
                    };

                    if !is_visible {
                        mask[my][mx] = FACE_NONE;
                        continue;
                    }

                    let texture_layer = block_info.get_texture_layer(voxel_id, face_idx);

                    let idx = voxel_id as usize;
                    let buffer_idx = if current_is_water { 2u8 } else {
                        if idx < block_info.render_modes.len()
                            && block_info.render_modes[idx] == RenderMode::Cutout
                        { 1 } else { 0 }
                    };

                    mask[my][mx] = texture_layer * 4 + buffer_idx as u32 + 1;
                }
            }

            greedy_merge_pass(
                face_idx, depth, depth_axis, mx_axis, my_axis,
                &mut mask, &block_info,
                &mut opaque_buf, &mut cutout_buf, &mut water_buf,
            );
        }
    }

    super::channel::MeshBuildResult {
        chunk_pos,
        opaque: opaque_buf,
        cutout: cutout_buf,
        water: water_buf,
    }
}

/// 将 mask 坐标解码为区块体素坐标
#[inline]
fn decode_mask_to_xyz(mx: usize, my: usize, depth: usize, depth_axis: usize, mx_axis: usize, my_axis: usize) -> (usize, usize, usize) {
    let mut coords = [0usize; 3];
    coords[depth_axis] = depth;
    coords[mx_axis] = mx;
    coords[my_axis] = my;
    (coords[0], coords[1], coords[2])
}

/// 对一个面切片执行贪心合并
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
            if face_key == FACE_NONE { mx += 1; continue; }

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

            let (positions, uvs) = get_merged_face_data(
                mx, my, depth, width, height,
                face_idx, depth_axis, mx_axis, my_axis,
                texture_layer, block_info.total_layers,
            );
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

/// 生成合并面的顶点坐标和 UV 坐标
fn get_merged_face_data(
    mx: usize, my: usize, depth: usize,
    width: usize, height: usize,
    face_idx: usize,
    depth_axis: usize, mx_axis: usize, my_axis: usize,
    texture_layer: u32,
    total_layers: u32,
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

    // UV（平铺式图集）
    let u0 = 0.0f32;
    let u1 = w / cs;
    let v0 = (texture_layer as f32 * cs) / (nt * cs);
    let v1 = (texture_layer as f32 * cs + h) / (nt * cs);

    match face_idx {
        0 => { // Top (Y+)
            let positions = [
                [base[0] + extent[0], base[1] + 1.0, base[2]],
                [base[0],             base[1] + 1.0, base[2]],
                [base[0],             base[1] + 1.0, base[2] + extent[2]],
                [base[0] + extent[0], base[1] + 1.0, base[2] + extent[2]],
            ];
            let uvs = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            (positions, uvs)
        }
        1 => { // Bottom (Y-)
            let positions = [
                [base[0],             base[1], base[2]],
                [base[0] + extent[0], base[1], base[2]],
                [base[0] + extent[0], base[1], base[2] + extent[2]],
                [base[0],             base[1], base[2] + extent[2]],
            ];
            let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
            (positions, uvs)
        }
        2 => { // Left (X-)
            let positions = [
                [base[0], base[1] + extent[1], base[2] + extent[2]],
                [base[0], base[1] + extent[1], base[2]],
                [base[0], base[1],             base[2]],
                [base[0], base[1],             base[2] + extent[2]],
            ];
            let uvs = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            (positions, uvs)
        }
        3 => { // Right (X+)
            let positions = [
                [base[0] + 1.0, base[1] + extent[1], base[2]],
                [base[0] + 1.0, base[1] + extent[1], base[2] + extent[2]],
                [base[0] + 1.0, base[1],             base[2] + extent[2]],
                [base[0] + 1.0, base[1],             base[2]],
            ];
            let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
            (positions, uvs)
        }
        4 => { // Front (Z+)
            let positions = [
                [base[0] + extent[0], base[1] + extent[1], base[2] + 1.0],
                [base[0],             base[1] + extent[1], base[2] + 1.0],
                [base[0],             base[1],             base[2] + 1.0],
                [base[0] + extent[0], base[1],             base[2] + 1.0],
            ];
            let uvs = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            (positions, uvs)
        }
        5 => { // Back (Z-)
            let positions = [
                [base[0],             base[1] + extent[1], base[2]],
                [base[0] + extent[0], base[1] + extent[1], base[2]],
                [base[0] + extent[0], base[1],             base[2]],
                [base[0],             base[1],             base[2]],
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
        neighbor_local_pos.x, neighbor_local_pos.y, neighbor_local_pos.z,
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
    if neighbor_voxel_id == 0 { return true; }
    if current_is_water { return false; }
    let nbr_is_solid = block_info.is_solid.get(neighbor_voxel_id as usize).copied().unwrap_or(true);
    !nbr_is_solid || neighbor_voxel_id == block_info.water_id
}
use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::world::chunk::ChunkData;
use crate::world::generation::biome::BiomeRegistry;
use crate::world::generation::context::ChunkGenContext;
use crate::world::generation::noise::GenerationBlockIds;
use crate::world::storage::WorldStorage;

/// 结构生成器 — 在地形生成后放置树木、矿脉等
pub struct StructureGenerator;

impl StructureGenerator {
    /// 在已生成的区块上放置结构
    /// 注意：跨区块边界的结构需要写入相邻区块（后续实现）
    pub fn generate_structures(
        chunk_data: &mut ChunkData,
        ctx: &ChunkGenContext,
        block_ids: &GenerationBlockIds,
        biome_registry: &BiomeRegistry,
        seed: u32,
        world_storage: &mut WorldStorage,
    ) {
        // 使用确定性随机（基于世界坐标和种子）
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let col = ctx.get_column(x, z);
                let biome = biome_registry.get(col.biome_index).unwrap();

                // 根据树木密度决定是否放树
                if biome.tree_density <= 0.0 { continue; }

                // 确定性哈希：同一位置总是产生相同结果
                let hash = simple_hash(col.world_x, col.world_z, seed);
                let chance = (hash & 0xFFFF) as f32 / 65536.0;

                if chance < biome.tree_density {
                    let local_surface_y = col.base_height - ctx.chunk_pos.y * CHUNK_SIZE as i32;

                    if local_surface_y < 0 || local_surface_y >= CHUNK_SIZE as i32 {
                        continue;
                    }

                    // 只在表面是草地上放树
                    let surface_id = chunk_data.get_voxel(x, local_surface_y as usize, z);
                    if surface_id != block_ids.grass {
                        continue;
                    }

                    // 转换为世界坐标
                    let base_world_x = col.world_x;
                    let base_world_y = col.base_height;
                    let base_world_z = col.world_z;

                    Self::place_tree(
                        chunk_data,
                        ctx.chunk_pos,
                        base_world_x,
                        base_world_y,
                        base_world_z,
                        block_ids,
                        world_storage,
                    );
                }
            }
        }
    }

    /// 放置一棵简单的树
    fn place_tree(
        chunk_data: &mut ChunkData,
        chunk_pos: IVec3,
        base_world_x: i32,
        base_world_y: i32,
        base_world_z: i32,
        block_ids: &GenerationBlockIds,
        world_storage: &mut WorldStorage,
    ) {
        let trunk_height = 4;

        // 树干
        for dy in 1..=trunk_height {
            let world_y = base_world_y + dy;
            set_voxel_world_aware(
                chunk_data, chunk_pos,
                base_world_x, world_y, base_world_z,
                block_ids.wood, world_storage,
            );
        }

        // 树冠（球形，中心在树顶 + 1）
        let crown_center_y = base_world_y + trunk_height + 1;
        let crown_radius = 2;

        for dx in -crown_radius..=crown_radius {
            for dy in -crown_radius..=crown_radius {
                for dz in -crown_radius..=crown_radius {
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq > crown_radius * crown_radius { continue; }

                    let wx = base_world_x + dx;
                    let wy = crown_center_y + dy;
                    let wz = base_world_z + dz;

                    set_voxel_world_aware(
                        chunk_data, chunk_pos,
                        wx, wy, wz,
                        block_ids.leaves, world_storage,
                    );
                }
            }
        }
    }
}

/// 简单确定性哈希（同一坐标+种子 → 同一值）
fn simple_hash(x: i32, z: i32, seed: u32) -> u32 {
    use std::num::Wrapping;
    let mut h = Wrapping(seed as u32);
    h ^= Wrapping(x as u32).0.wrapping_mul(0x45d9f3b);
    h ^= Wrapping(z as u32).0.wrapping_mul(0x1f83d9ab);
    h ^= h >> 16;
    h.0
}

// 设置方块
fn set_voxel_world_aware(
    chunk_data: &mut ChunkData,
    current_chunk_pos: IVec3,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    block_id: u16,
    world_storage: &mut WorldStorage,
) {
    let target_chunk = IVec3::new(
        world_x.div_euclid(CHUNK_SIZE as i32),
        world_y.div_euclid(CHUNK_SIZE as i32),
        world_z.div_euclid(CHUNK_SIZE as i32),
    );
    let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;

    if target_chunk == current_chunk_pos {
        // 当前区块
        if chunk_data.get_voxel(local_x, local_y, local_z) == 0 {
            chunk_data.set_voxel(local_x, local_y, local_z, block_id);
        }
    } else {
        // 相邻区块
        if let Some(neighbor) = world_storage.loaded_chunks.get_mut(&target_chunk) {
            if neighbor.get_voxel(local_x, local_y, local_z) == 0 {
                neighbor.set_voxel(local_x, local_y, local_z, block_id);
            }
        }
        // 相邻区块尚未加载时，跳过
    }
}
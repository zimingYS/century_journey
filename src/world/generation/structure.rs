use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::world::chunk::ChunkData;
use crate::world::generation::biome::BiomeRegistry;
use crate::world::generation::context::ChunkGenContext;
use crate::world::generation::noise::GenerationBlockIds;

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

                    Self::place_tree(chunk_data, x, local_surface_y, z, block_ids);
                }
            }
        }
    }

    /// 放置一棵简单的树
    fn place_tree(
        chunk_data: &mut ChunkData,
        base_x: usize,
        base_y: i32,
        base_z: usize,
        block_ids: &GenerationBlockIds,
    ) {
        let trunk_height = 4;

        // 树干
        for dy in 1..=trunk_height {
            let y = (base_y + dy) as usize;
            if y >= CHUNK_SIZE { break; }
            chunk_data.set_voxel(base_x, y, base_z, block_ids.wood);
        }

        // 树冠（球形，中心在树顶 + 1）
        let crown_center_y = (base_y + trunk_height + 1) as i32;
        let crown_radius = 2;

        for dx in -crown_radius..=crown_radius {
            for dy in -crown_radius..=crown_radius {
                for dz in -crown_radius..=crown_radius {
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq > crown_radius * crown_radius { continue; }

                    let lx = base_x as i32 + dx;
                    let ly = crown_center_y + dy;
                    let lz = base_z as i32 + dz;

                    if lx < 0 || lx >= CHUNK_SIZE as i32 { continue; }
                    if ly < 0 || ly >= CHUNK_SIZE as i32 { continue; }
                    if lz < 0 || lz >= CHUNK_SIZE as i32 { continue; }

                    let current = chunk_data.get_voxel(lx as usize, ly as usize, lz as usize);
                    if current == block_ids.air {
                        chunk_data.set_voxel(lx as usize, ly as usize, lz as usize, block_ids.leaves);
                    }
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
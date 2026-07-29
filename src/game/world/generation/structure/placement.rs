use std::collections::HashMap;
use std::sync::Arc;

/// 结构生成任务的局部输入与延迟写入结果。
pub struct StructureGenerationWorkspace {
    loaded_chunks: HashMap<IVec3, Arc<ChunkData>>,
    pending_writes: PendingVoxelWrites,
}

impl StructureGenerationWorkspace {
    pub fn new(loaded_chunks: HashMap<IVec3, Arc<ChunkData>>) -> Self {
        Self {
            loaded_chunks,
            pending_writes: PendingVoxelWrites::default(),
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        HashMap<IVec3, Arc<ChunkData>>,
        HashMap<IVec3, Vec<PendingVoxel>>,
    ) {
        (self.loaded_chunks, self.pending_writes.writes)
    }
}

/// 结构生成器 — 在地形生成后放置树木、矿脉等
pub struct StructureGenerator;

impl StructureGenerator {
    /// 在已生成的区块上放置结构
    pub fn generate_structures_world_aware(
        chunk_pos: IVec3,
        ctx: &ChunkGenContext,
        block_ids: &GenerationBlockIds,
        biome_registry: &BiomeRegistry,
        seed: u32,
        workspace: &mut StructureGenerationWorkspace,
    ) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let col = ctx.get_column(x, z);
                let biome = biome_registry.get(col.biome_index).unwrap();

                if biome.tree_density <= 0.0 {
                    continue;
                }

                let hash = simple_hash(col.world_x, col.world_z, seed);
                let chance = (hash & 0xFFFF) as f32 / 65536.0;
                if chance >= biome.tree_density {
                    continue;
                }

                let base_world_x = col.world_x;
                let base_world_y = col.base_height;
                let base_world_z = col.world_z;

                // 必须已经存在区块数据
                let Some(surface_id) =
                    Self::get_world_voxel(base_world_x, base_world_y, base_world_z, workspace)
                else {
                    continue;
                };

                // 只允许符合标签的方块上生成树
                if !block_ids.is_tree_plantable(surface_id) {
                    continue;
                }

                Self::place_tree_world_aware(
                    chunk_pos,
                    base_world_x,
                    base_world_y,
                    base_world_z,
                    block_ids,
                    workspace,
                );
            }
        }
    }

    /// 放置一棵简单的树
    fn place_tree_world_aware(
        chunk_pos: IVec3,
        base_world_x: i32,
        base_world_y: i32,
        base_world_z: i32,
        block_ids: &GenerationBlockIds,
        workspace: &mut StructureGenerationWorkspace,
    ) {
        let hash = simple_hash(base_world_x, base_world_z, 114514);
        let trunk_height = 4 + (hash % 3) as i32;
        let crown_radius = 2 + ((hash >> 8) % 2) as i32;
        let crown_center_y = base_world_y + trunk_height + 1;

        // 树干
        for dy in 1..=trunk_height {
            let wy = base_world_y + dy;
            set_voxel_world_aware(
                chunk_pos,
                base_world_x,
                wy,
                base_world_z,
                block_ids.wood,
                workspace,
            );
        }

        // 树冠
        for dx in -crown_radius..=crown_radius {
            for dy in -crown_radius..=crown_radius {
                for dz in -crown_radius..=crown_radius {
                    if dx * dx + dy * dy + dz * dz > crown_radius * crown_radius {
                        continue;
                    }
                    let wx = base_world_x + dx;
                    let wy = crown_center_y + dy;
                    let wz = base_world_z + dz;
                    set_voxel_world_aware(chunk_pos, wx, wy, wz, block_ids.leaves, workspace);
                }
            }
        }
    }

    pub fn get_world_voxel(
        world_x: i32,
        world_y: i32,
        world_z: i32,
        workspace: &StructureGenerationWorkspace,
    ) -> Option<u16> {
        let chunk_pos = IVec3::new(
            world_x.div_euclid(CHUNK_SIZE as i32),
            world_y.div_euclid(CHUNK_SIZE as i32),
            world_z.div_euclid(CHUNK_SIZE as i32),
        );

        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;

        workspace
            .loaded_chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.get_voxel(local_x, local_y, local_z))
    }
}

/// 简单确定性哈希
fn simple_hash(x: i32, z: i32, seed: u32) -> u32 {
    use std::num::Wrapping;
    let mut h = Wrapping(seed);
    h ^= Wrapping(x as u32).0.wrapping_mul(0x45d9f3b);
    h ^= Wrapping(z as u32).0.wrapping_mul(0x1f83d9ab);
    h ^= h >> 16;
    h.0
}

// 设置方块
pub fn set_voxel_world_aware(
    base_chunk_pos: IVec3,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    block_id: u16,
    workspace: &mut StructureGenerationWorkspace,
) {
    // 计算目标区块坐标
    let target_chunk = IVec3::new(
        world_x.div_euclid(CHUNK_SIZE as i32),
        world_y.div_euclid(CHUNK_SIZE as i32),
        world_z.div_euclid(CHUNK_SIZE as i32),
    );

    let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;

    if target_chunk == base_chunk_pos {
        // 当前区块直接写入
        if let Some(arc) = workspace.loaded_chunks.get_mut(&target_chunk) {
            let chunk_data = Arc::make_mut(arc);
            if chunk_data.get_voxel(local_x, local_y, local_z) == 0 {
                chunk_data.set_voxel(local_x, local_y, local_z, block_id);
            }
        }
    } else {
        // 跨区块写入
        if let Some(arc) = workspace.loaded_chunks.get_mut(&target_chunk) {
            let neighbor = Arc::make_mut(arc);
            if neighbor.get_voxel(local_x, local_y, local_z) == 0 {
                neighbor.set_voxel(local_x, local_y, local_z, block_id);
            }
        } else {
            // 目标区块未加载，加入延迟写入队列
            workspace
                .pending_writes
                .writes
                .entry(target_chunk)
                .or_default()
                .push(PendingVoxel {
                    local_x,
                    local_y,
                    local_z,
                    block_id,
                });
        }
    }
}
use crate::content::biome::BiomeRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::block_ids::GenerationBlockIds;
use crate::game::world::generation::structure::pending_writes::{PendingVoxel, PendingVoxelWrites};
use crate::game::world::generation::terrain::context::ChunkGenContext;
use bevy::prelude::IVec3;

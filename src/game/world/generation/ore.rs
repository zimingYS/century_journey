//! 矿石矿脉投放：在地形生成后，用确定性 3D 噪声把石头替换为矿石。

use crate::content::ore_vein::registry::RuntimeOreVein;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::terrain::noise::NoiseSampler;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::math::IVec3;
use noise::NoiseFn;

/// 在基础地形之上投放矿脉：仅替换石头，其余方块不动。
///
/// `veins` 已按优先级从高到低排序，命中后立即停止，保证稀有矿优先。
pub fn apply_ores(
    chunk_data: &mut ChunkData,
    chunk_pos: IVec3,
    noise: &NoiseSampler,
    stone_id: u16,
    veins: &[RuntimeOreVein],
) {
    let start = chunk_pos * CHUNK_SIZE as i32;
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let world_pos = start + IVec3::new(x as i32, y as i32, z as i32);
                if chunk_data.get_voxel(x, y, z) != stone_id {
                    continue; // 只替换石头
                }
                for vein in veins {
                    let v = &vein.definition;
                    if world_pos.y < v.min_y || world_pos.y > v.max_y {
                        continue;
                    }
                    let n = noise.ore.get([
                        world_pos.x as f64 * v.scale,
                        world_pos.y as f64 * v.scale,
                        world_pos.z as f64 * v.scale,
                    ]);
                    if n < v.threshold {
                        chunk_data.set_voxel(x, y, z, vein.block_id);
                        break; // 已放矿，不再被低优先级矿覆盖
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/generation/ore.rs"]
mod tests;

//! 洞穴雕刻：地形生成后、矿石投放前，用 3D 噪声把深部石头挖空。

use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::terrain::context::ChunkGenContext;
use crate::game::world::generation::terrain::noise::NoiseSampler;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::math::IVec3;
use noise::NoiseFn;

/// 洞穴雕刻参数（v1 常量表；手感稳定后可按矿脉模式数据驱动）。
pub struct CaveProfile {
    /// 3D 噪声阈值：低于阈值挖空（值越低洞穴越少）。
    pub threshold: f64,
    /// 噪声世界坐标缩放（越小洞穴越大）。
    pub scale: f64,
    /// 最小世界高度（含），低于此不挖（保护深层基岩）。
    pub min_y: i32,
    /// 距地表的最小天花板厚度（方块数），防止天窗直通地表。
    pub min_ceiling: i32,
}

/// 默认洞穴剖面（v1 调参起点）。
pub const DEFAULT_CAVE_PROFILE: CaveProfile = CaveProfile {
    threshold: -0.30,
    scale: 0.05,
    min_y: -40,
    min_ceiling: 4,
};

/// 在地形生成后雕刻洞穴：只挖石头，其余方块不动。
pub fn apply_caves(
    chunk_data: &mut ChunkData,
    ctx: &ChunkGenContext,
    noise: &NoiseSampler,
    stone_id: u16,
    profile: &CaveProfile,
) {
    let start = ctx.chunk_pos * CHUNK_SIZE as i32;
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            // 每列有自己的地表高度，天花板保护按列计算，不挖穿地形
            let surface_y = ctx.get_column(x, z).base_height;
            for y in 0..CHUNK_SIZE {
                let world_pos = start + IVec3::new(x as i32, y as i32, z as i32);
                if chunk_data.get_voxel(x, y, z) != stone_id {
                    continue; // 只挖石头
                }
                if world_pos.y < profile.min_y {
                    continue; // 深度保护
                }
                if world_pos.y >= surface_y - profile.min_ceiling {
                    continue; // 天花板保护：地表下 min_ceiling 层不挖
                }
                let n = noise.cave.get([
                    world_pos.x as f64 * profile.scale,
                    world_pos.y as f64 * profile.scale,
                    world_pos.z as f64 * profile.scale,
                ]);
                if n < profile.threshold {
                    chunk_data.set_voxel(x, y, z, 0); // air
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/generation/cave.rs"]
mod tests;

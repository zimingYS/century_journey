//! 把权威光级数组烘焙成网格顶点光色（乘法衰减系数语义）。

use std::sync::Arc;

use bevy::math::IVec3;

use crate::client::renderer::world::DIRECTIONS;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell, LightRgb};
use crate::shared::voxel::CHUNK_SIZE;

/// 光照尚未就绪时使用的中性临时环境光，保证新区块纹理不会先显示成纯黑。
const PENDING_LIGHT_CELL: LightCell = LightCell {
    sky: LightRgb { r: 6, g: 6, b: 6 },
    block: LightRgb { r: 0, g: 0, b: 0 },
};

/// 按世界坐标采样光级；跨区块边界回退到邻居快照，缺失使用中性临时环境光。
pub fn sample_light_at(
    world_pos: IVec3,
    chunk_pos: IVec3,
    light: &Option<Arc<ChunkLight>>,
    neighbor_lights: &[Option<Arc<ChunkLight>>; 6],
) -> LightCell {
    let (target_chunk, local) = split_world(world_pos);
    let fetch = |light: &Option<Arc<ChunkLight>>| {
        light
            .as_ref()
            .map(|l| l.get(local.x as usize, local.y as usize, local.z as usize))
    };
    if target_chunk == chunk_pos {
        return fetch(light).unwrap_or(PENDING_LIGHT_CELL);
    }
    let delta = target_chunk - chunk_pos;
    for (i, (dir, _)) in DIRECTIONS.iter().enumerate() {
        if *dir == delta {
            return fetch(&neighbor_lights[i]).unwrap_or(PENDING_LIGHT_CELL);
        }
    }
    PENDING_LIGHT_CELL
}

/// 光级 → 顶点光色（RGBA）。
///
/// 语义：乘法衰减系数 —— 露天（sky 高）处为白（不改变贴图），
/// 无光处趋近黑（贴图被压暗），方块光按通道参与并保留光源色温。
#[inline]
pub fn light_to_color(cell: LightCell) -> [f32; 4] {
    light_rgb_to_color(cell.combined())
}

/// 把已经合并的 4bit RGB 光级转换为线性顶点光色。
#[inline]
pub fn light_rgb_to_color(light: LightRgb) -> [f32; 4] {
    [
        light.r as f32 / 15.0,
        light.g as f32 / 15.0,
        light.b as f32 / 15.0,
        1.0,
    ]
}

/// 把世界坐标拆成区块坐标与区块内局部坐标。
#[inline]
fn split_world(pos: IVec3) -> (IVec3, IVec3) {
    (
        IVec3::new(
            pos.x.div_euclid(CHUNK_SIZE as i32),
            pos.y.div_euclid(CHUNK_SIZE as i32),
            pos.z.div_euclid(CHUNK_SIZE as i32),
        ),
        IVec3::new(
            pos.x.rem_euclid(CHUNK_SIZE as i32),
            pos.y.rem_euclid(CHUNK_SIZE as i32),
            pos.z.rem_euclid(CHUNK_SIZE as i32),
        ),
    )
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/lighting/bake.rs"]
mod tests;

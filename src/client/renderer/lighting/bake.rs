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
/// 完全无光时保留的中性纹理可读下限；只作用于表现，不参与传播。
const MIN_DARK_SURFACE_LIGHT: f32 = 0.08;
/// 4bit 光级的感知亮度查找表；提升中低光级，同时保持满光级不变。
const PERCEPTUAL_LIGHT_LEVELS: [f32; 16] = [
    0.0, 0.172_005, 0.269_905, 0.351_293, 0.423_526, 0.489_634, 0.551_238, 0.609_333, 0.664_583,
    0.717_461, 0.768_317, 0.817_421, 0.864_985, 0.911_179, 0.956_145, 1.0,
];

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
/// 无光处保留很低的中性可读度，方块光按通道参与并保留光源色温。
#[inline]
pub fn light_to_color(cell: LightCell) -> [f32; 4] {
    light_rgb_to_color(cell.combined())
}

/// 把独立方块 RGB 光级编码到第二组顶点 UV。
///
/// 4bit RGB 共占 12bit，`f32` 能无损保存该整数；第二分量保留为零，供区块材质在
/// GPU 中恢复方块光颜色。天空光仍由顶点色携带，两者不能提前合并，否则远景材质
/// 无法区分自然光和需要自发光补偿的方块光。
#[inline]
pub fn block_light_to_uv(light: LightRgb) -> [f32; 2] {
    let packed = ((u16::from(light.r) & 0xF) << 8)
        | ((u16::from(light.g) & 0xF) << 4)
        | (u16::from(light.b) & 0xF);
    [f32::from(packed), 0.0]
}

/// 把已经合并的 4bit RGB 光级转换为线性顶点光色。
///
/// 曲线只由最亮通道决定增益，随后等比缩放 RGB，避免对每个通道分别提亮后
/// 改变暖光、染色玻璃或多光源混色的色相。
#[inline]
pub fn light_rgb_to_color(light: LightRgb) -> [f32; 4] {
    let peak = light.r.max(light.g).max(light.b).min(15);
    let gain = if peak == 0 {
        1.0
    } else {
        PERCEPTUAL_LIGHT_LEVELS[peak as usize] / (peak as f32 / 15.0)
    };
    let channel =
        |value: u8| (value.min(15) as f32 / 15.0 * gain).clamp(MIN_DARK_SURFACE_LIGHT, 1.0);
    [channel(light.r), channel(light.g), channel(light.b), 1.0]
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

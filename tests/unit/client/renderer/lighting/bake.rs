use super::*;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell};
use bevy::math::IVec3;
use std::sync::Arc;

fn no_neighbors() -> [Option<Arc<ChunkLight>>; 6] {
    std::array::from_fn(|_| None)
}

fn rgb(r: u8, g: u8, b: u8) -> LightRgb {
    LightRgb { r, g, b }
}

fn cell(sky: LightRgb, block: LightRgb) -> LightCell {
    LightCell { sky, block }
}

#[test]
fn light_to_color_uses_sky_as_white_fallback() {
    // 露天（sky 15）：白色，不改变贴图。
    assert_eq!(
        light_to_color(cell(rgb(15, 15, 15), rgb(0, 0, 0))),
        [1.0, 1.0, 1.0, 1.0]
    );
    // 洞穴无光：保持昏暗，但不能把纹理乘成纯黑。
    let dark = light_to_color(cell(rgb(0, 0, 0), rgb(0, 0, 0)));
    assert_eq!(
        dark,
        [
            MIN_DARK_SURFACE_LIGHT,
            MIN_DARK_SURFACE_LIGHT,
            MIN_DARK_SURFACE_LIGHT,
            1.0,
        ]
    );
    // 洞穴火把：暖色保留。
    let torch = light_to_color(cell(rgb(0, 0, 0), rgb(15, 9, 4)));
    assert_eq!(torch[0], 1.0);
    assert!(torch[1] > torch[2], "暖白光应 R > B");
}

#[test]
fn perceptual_curve_lifts_distance_without_changing_hue() {
    let color = light_rgb_to_color(rgb(8, 4, 2));

    assert!(color[0] > 8.0 / 15.0, "中光级应比线性映射更清晰");
    assert!((color[0] / color[1] - 2.0).abs() < 0.001);
    assert!((color[1] / color[2] - 2.0).abs() < 0.001);
}

#[test]
fn block_light_uv_keeps_all_twelve_rgb_bits() {
    assert_eq!(block_light_to_uv(rgb(15, 9, 4)), [0xF94 as f32, 0.0]);
    assert_eq!(block_light_to_uv(rgb(0, 0, 0)), [0.0, 0.0]);
}

#[test]
fn sample_light_at_falls_back_to_neighbor_chunk() {
    let chunk_pos = IVec3::ZERO;
    let mut own = ChunkLight::default();
    own.set(8, 8, 8, cell(rgb(0, 0, 0), rgb(5, 0, 0)));
    let mut neighbor = ChunkLight::default();
    neighbor.set(0, 3, 4, cell(rgb(0, 0, 0), rgb(0, 7, 0)));

    // 本区块内采样。
    let inside = sample_light_at(
        IVec3::new(8, 8, 8),
        chunk_pos,
        &Some(Arc::new(own.clone())),
        &no_neighbors(),
    );
    assert_eq!(inside.block.r, 5);

    // 跨区块边界：世界坐标 (16, 3, 4) 属于 +X 邻居，其局部坐标 (0, 3, 4)。
    let mut neighbor_lights = no_neighbors();
    neighbor_lights[3] = Some(Arc::new(neighbor.clone())); // +X 索引
    let outside = sample_light_at(
        IVec3::new(16, 3, 4),
        chunk_pos,
        &Some(Arc::new(own)),
        &neighbor_lights,
    );
    assert_eq!(outside.block.g, 7);

    // 缺失邻居：使用中性临时环境光，避免等待光照时贴图纯黑。
    let missing = sample_light_at(
        IVec3::new(16, 3, 4),
        chunk_pos,
        &Some(Arc::new(ChunkLight::default())),
        &no_neighbors(),
    );
    assert_eq!(missing.block.r, 0);
    assert_eq!(missing.sky, rgb(6, 6, 6));
}

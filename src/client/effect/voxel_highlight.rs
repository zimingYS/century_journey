//! 根据玩家瞄准结果显示方块选择线框。

use crate::game::player::interaction::targeting::TargetVoxel;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::{Gizmos, Res, Time, Transform};

/// 绘制方块高亮框系统
pub fn draw_voxel_highlight_system(
    time: Res<Time>,
    target_voxel: Res<TargetVoxel>,
    mut gizmos: Gizmos,
) {
    if let Some(ray_result) = &target_voxel.result {
        let center = Vec3::new(
            ray_result.hit_pos.x as f32 + 0.5,
            ray_result.hit_pos.y as f32 + 0.5,
            ray_result.hit_pos.z as f32 + 0.5,
        );

        let pulse = (time.elapsed_secs() * 3.2).sin() * 0.5 + 0.5;
        let scale = 1.006 + pulse * 0.008;
        gizmos.cube(
            Transform::from_translation(center).with_scale(Vec3::splat(scale)),
            Color::srgba(0.78 + pulse * 0.16, 0.93, 1.0, 0.88),
        );
    }
}

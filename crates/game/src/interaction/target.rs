use crate::interaction::world_query::world_is_solid;
use bevy::prelude::*;
use cj_core::pos::BlockPos;
use cj_runtime::world_data::WorldData;

const REACH_DISTANCE: f32 = 8.0;

#[derive(Resource, Default)]
pub struct PlayerTarget {
    pub block: Option<BlockPos>,
}

pub fn raycast<F>(origin: Vec3, direction: Vec3, max_distance: f32, is_solid: F) -> Option<BlockPos>
where
    F: Fn(BlockPos) -> bool,
{
    let len = direction.length();
    if len == 0.0 {
        return None;
    }
    let dir = direction / len;

    // 计算起点所在方块
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    // 三个轴各自的DDA
    let (step_x, mut t_max_x, t_delta_x) = dda_axis(origin.x, dir.x, x);
    let (step_y, mut t_max_y, t_delta_y) = dda_axis(origin.y, dir.y, y);
    let (step_z, mut t_max_z, t_delta_z) = dda_axis(origin.z, dir.z, z);

    // 前进检测
    loop {
        let t;
        if t_max_x < t_max_y && t_max_x < t_max_z {
            x += step_x;
            t = t_max_x;
            t_max_x += t_delta_x;
        } else if t_max_y < t_max_z {
            y += step_y;
            t = t_max_y;
            t_max_y += t_delta_y;
        } else {
            z += step_z;
            t = t_max_z;
            t_max_z += t_delta_z;
        }

        if t > max_distance {
            return None;
        }

        let pos = BlockPos { x, y, z };
        if is_solid(pos) {
            return Some(pos);
        }
    }
}

/// 计算单个坐标轴上的DDA
#[inline]
fn dda_axis(origin: f32, dir: f32, cell: i32) -> (i32, f32, f32) {
    if dir > 0.0 {
        let boundary = cell as f32 + 1.0;
        (1, (boundary - origin) / dir, 1.0 / dir)
    } else if dir < 0.0 {
        let boundary = cell as f32;
        (-1, (origin - boundary) / -dir, 1.0 / -dir)
    } else {
        (0, f32::INFINITY, f32::INFINITY)
    }
}

pub fn update_player_target(
    world: Res<WorldData>,
    camera: Query<&GlobalTransform, With<Camera3d>>,
    mut target: ResMut<PlayerTarget>,
) {
    let Ok(global) = camera.single() else {
        return;
    };

    let origin = global.translation();
    let direction = global.rotation() * Vec3::NEG_Z;

    target.block = raycast(origin, direction, REACH_DISTANCE, |pos| {
        world_is_solid(&world, pos)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 直行命中
    #[test]
    fn straight_hit() {
        // 设置起点、方向和实心方块
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let target = BlockPos::new(3, 0, 0);

        // 执行射线检测
        let result = raycast(origin, direction, 8.0, |pos| pos == target);

        // 验证命中目标方块
        assert_eq!(result, Some(target));
    }

    /// 距离截断
    #[test]
    fn distance_limit() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let target = BlockPos::new(10, 0, 0);

        let result = raycast(origin, direction, 5.0, |pos| pos == target);

        assert_eq!(result, None);
    }

    /// 起点格不自选
    #[test]
    fn origin_cell_is_not_targeted() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let start = BlockPos::new(0, 0, 0);

        let result = raycast(origin, direction, 8.0, |pos| pos == start);

        assert_eq!(result, None);
    }

    /// 最近优先
    #[test]
    fn nearest_hit_first() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let nearest = BlockPos::new(2, 0, 0);
        let farther = BlockPos::new(5, 0, 0);

        let result = raycast(origin, direction, 8.0, |pos| {
            pos == nearest || pos == farther
        });

        assert_eq!(result, Some(nearest));
    }

    /// 零方向
    #[test]
    fn zero_direction() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(0.0, 0.0, 0.0);
        let target = BlockPos::new(3, 0, 0);

        let result = raycast(origin, direction, 8.0, |pos| pos == target);

        assert_eq!(result, None);
    }
}

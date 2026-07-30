//! 从权威玩家朝向计算交互射线，并查询最近体素目标。

use crate::game::world::chunk::ChunkData;
use crate::game::world::state::WorldState;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::math::{IVec3, Quat, UVec3, Vec3};
use bevy::prelude::{Query, Res, ResMut, Resource, Transform, With};

/// 交互射线起点相对玩家实体原点的眼部高度。
pub const PLAYER_EYE_HEIGHT: f32 = 0.78;
const PLAYER_RAY_FORWARD_OFFSET: f32 = 0.24;

#[derive(Debug)]
/// 体素 DDA 射线检测得到的最近命中信息。
pub struct RaycastResult {
    /// 击中的方块的世界绝对坐标
    pub hit_pos: IVec3,
    /// 击中面的法线
    pub normal: IVec3,
    /// 击中方块所在的区块世界坐标
    pub chunk_pos: IVec3,
    /// 击中方块在区块内部的局部坐标
    pub local_pos: UVec3,
}

#[derive(Resource, Default, Debug)]
/// 保存当前固定步内玩家最近的可交互体素目标。
pub struct TargetVoxel {
    /// 存储当前帧射线是否击中了方块
    pub result: Option<RaycastResult>,
}

/// 使用权威玩家位置和瞄准角刷新当前体素目标。
pub fn update_raycast_system(
    world_state: Res<WorldState>,
    player_query: Query<
        (
            &Transform,
            &crate::game::player::movement::components::PlayerAim,
        ),
        With<crate::game::player::identity::Player>,
    >,
    mut target_voxel: ResMut<TargetVoxel>,
) {
    let Ok((player_transform, aim)) = player_query.single() else {
        target_voxel.result = None;
        return;
    };

    let (origin, direction) = player_interaction_ray(player_transform, aim.pitch);

    target_voxel.result = raycast_voxel(&origin, &direction, &world_state, 0.0);
}

/// 根据玩家身体朝向和俯仰角构造世界空间交互射线。
pub fn player_interaction_ray(player_transform: &Transform, pitch: f32) -> (Vec3, Vec3) {
    let player_rotation = player_transform.rotation;
    let horizontal_forward = player_rotation * Vec3::NEG_Z;
    let origin = player_transform.translation
        + Vec3::Y * PLAYER_EYE_HEIGHT
        + horizontal_forward * PLAYER_RAY_FORWARD_OFFSET;
    let direction = player_rotation * Quat::from_rotation_x(pitch.clamp(-1.5, 1.5)) * Vec3::NEG_Z;
    (origin, direction.normalize())
}

/// 使用三维 DDA 遍历体素，返回八格范围内第一个非空气方块。
pub fn raycast_voxel(
    origin: &Vec3,
    direction: &Vec3,
    world_state: &WorldState,
    start_offset: f32,
) -> Option<RaycastResult> {
    let max_distance = 8.0;
    let pos = *origin + *direction * start_offset;
    let mut x = pos.x.floor() as i32;
    let mut y = pos.y.floor() as i32;
    let mut z = pos.z.floor() as i32;

    // DDA 为每条轴预计算前进符号、跨越单个体素的参数距离和首个边界距离。
    let (step_x, step_y, step_z) = (
        if direction.x > 0.0 { 1 } else { -1 },
        if direction.y > 0.0 { 1 } else { -1 },
        if direction.z > 0.0 { 1 } else { -1 },
    );

    let (t_delta_x, t_delta_y, t_delta_z) = (
        if direction.x != 0.0 {
            1.0 / direction.x.abs()
        } else {
            f32::MAX
        },
        if direction.y != 0.0 {
            1.0 / direction.y.abs()
        } else {
            f32::MAX
        },
        if direction.z != 0.0 {
            1.0 / direction.z.abs()
        } else {
            f32::MAX
        },
    );

    let mut t_max_x = calculate_t_max(pos.x, x, step_x, t_delta_x);
    let mut t_max_y = calculate_t_max(pos.y, y, step_y, t_delta_y);
    let mut t_max_z = calculate_t_max(pos.z, z, step_z, t_delta_z);

    let mut distance = 0.0;
    let mut last_normal = IVec3::ZERO;

    while distance < max_distance {
        if let Some((chunk_pos, local_pos)) = check_voxel(x, y, z, world_state) {
            return Some(RaycastResult {
                hit_pos: IVec3::new(x, y, z),
                normal: last_normal,
                chunk_pos,
                local_pos,
            });
        }

        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                last_normal = IVec3::new(-step_x, 0, 0);
                x += step_x;
                distance = t_max_x;
                t_max_x += t_delta_x;
            } else {
                last_normal = IVec3::new(0, 0, -step_z);
                z += step_z;
                distance = t_max_z;
                t_max_z += t_delta_z;
            }
        } else if t_max_y < t_max_z {
            last_normal = IVec3::new(0, -step_y, 0);
            y += step_y;
            distance = t_max_y;
            t_max_y += t_delta_y;
        } else {
            last_normal = IVec3::new(0, 0, -step_z);
            z += step_z;
            distance = t_max_z;
            t_max_z += t_delta_z;
        }

        if !is_valid_height(y) {
            return None;
        }
    }

    None
}

/// 计算射线从当前位置抵达指定轴下一个体素边界的参数距离。
fn calculate_t_max(pos: f32, voxel_coord: i32, step: i32, t_delta: f32) -> f32 {
    if step > 0 {
        ((voxel_coord + 1) as f32 - pos) * t_delta
    } else {
        (pos - voxel_coord as f32) * t_delta
    }
}

fn is_valid_height(y: i32) -> bool {
    (-64..256).contains(&y)
}

fn check_voxel(x: i32, y: i32, z: i32, world_state: &WorldState) -> Option<(IVec3, UVec3)> {
    let chunk_pos = IVec3::new(
        x.div_euclid(CHUNK_SIZE as i32),
        y.div_euclid(CHUNK_SIZE as i32),
        z.div_euclid(CHUNK_SIZE as i32),
    );

    let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

    if let Some(chunk_data) = world_state.chunk(chunk_pos) {
        let voxel_id = chunk_data.voxels[ChunkData::xyz_to_index(local_x, local_y, local_z)];

        if voxel_id != 0u16 {
            return Some((
                chunk_pos,
                UVec3::new(local_x as u32, local_y as u32, local_z as u32),
            ));
        }
    }

    None
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/interaction/targeting.rs"]
mod tests;

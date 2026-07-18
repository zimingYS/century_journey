use crate::content::constant::world::CHUNK_SIZE;
use crate::game::world::chunk::ChunkData;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

const PLAYER_EYE_HEIGHT: f32 = 0.78;
const PLAYER_RAY_FORWARD_OFFSET: f32 = 0.24;

#[derive(Debug)]
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
pub struct TargetVoxel {
    /// 存储当前帧射线是否击中了方块
    pub result: Option<RaycastResult>,
}

pub fn update_raycast_system(
    world_storage: Res<WorldStorage>,
    camera_query: Query<&crate::shared::components::FpsCamera>,
    player_query: Query<&GlobalTransform, With<crate::game::player::components::Player>>,
    mut target_voxel: ResMut<TargetVoxel>,
) {
    let (Ok(camera), Ok(player_transform)) = (camera_query.single(), player_query.single()) else {
        target_voxel.result = None;
        return;
    };

    let (origin, direction) = player_interaction_ray(player_transform, camera.pitch);

    target_voxel.result = raycast_voxel(&origin, &direction, &world_storage, 0.0);
}

fn player_interaction_ray(player_transform: &GlobalTransform, pitch: f32) -> (Vec3, Vec3) {
    let player_rotation = player_transform.rotation();
    let horizontal_forward = player_rotation * Vec3::NEG_Z;
    let origin = player_transform.translation()
        + Vec3::Y * PLAYER_EYE_HEIGHT
        + horizontal_forward * PLAYER_RAY_FORWARD_OFFSET;
    let direction = player_rotation * Quat::from_rotation_x(pitch.clamp(-1.5, 1.5)) * Vec3::NEG_Z;
    (origin, direction.normalize())
}

pub fn raycast_voxel(
    origin: &Vec3,    // 射线起点
    direction: &Vec3, // 射线方向
    world_storage: &WorldStorage,
    start_offset: f32, // 起点偏移量
) -> Option<RaycastResult> {
    // 最大射线距离
    let max_distance = 8.0;

    // 计算射线实际起点
    let pos = *origin + *direction * start_offset;

    // 计算起点坐标
    let mut x = pos.x.floor() as i32;
    let mut y = pos.y.floor() as i32;
    let mut z = pos.z.floor() as i32;

    // 计算射线前进方向
    let (step_x, step_y, step_z) = (
        if direction.x > 0.0 { 1 } else { -1 },
        if direction.y > 0.0 { 1 } else { -1 },
        if direction.z > 0.0 { 1 } else { -1 },
    );

    // 计算DDA
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
        if let Some((chunk_pos, local_pos)) = check_voxel(x, y, z, world_storage) {
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

// 计算到下一个方块边界的初始值
fn calculate_t_max(pos: f32, voxel_coord: i32, step: i32, t_delta: f32) -> f32 {
    if step > 0 {
        ((voxel_coord + 1) as f32 - pos) * t_delta
    } else {
        (pos - voxel_coord as f32) * t_delta
    }
}

// 限制垂直高度判断
fn is_valid_height(y: i32) -> bool {
    (-64..256).contains(&y)
}

// 检查方块
fn check_voxel(x: i32, y: i32, z: i32, world_storage: &WorldStorage) -> Option<(IVec3, UVec3)> {
    // 换算出该绝对坐标所对应的区块世界坐标
    let chunk_pos = IVec3::new(
        x.div_euclid(CHUNK_SIZE as i32),
        y.div_euclid(CHUNK_SIZE as i32),
        z.div_euclid(CHUNK_SIZE as i32),
    );

    // 换算出在该区块内部的局部坐标
    let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

    if let Some(chunk_data) = world_storage.loaded_chunks.get(&chunk_pos) {
        let voxel_id = chunk_data.voxels[ChunkData::xyz_to_index(local_x, local_y, local_z)];

        // 只要不是空气，一律视为“被撞击的实体”
        if voxel_id != 0u16 {
            return Some((
                chunk_pos,
                UVec3::new(local_x as u32, local_y as u32, local_z as u32),
            ));
        }
    }

    None
}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_ray_starts_at_player_and_uses_player_facing() {
        let player = GlobalTransform::from(
            Transform::from_xyz(10.0, 20.0, 30.0)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        );
        let (origin, direction) = player_interaction_ray(&player, 0.0);

        assert!((origin.y - (20.0 + PLAYER_EYE_HEIGHT)).abs() < 0.0001);
        assert!(direction.x < -0.99);
        assert!(origin.x < 10.0);
    }

    #[test]
    fn interaction_ray_pitch_changes_direction_without_moving_to_camera() {
        let player = GlobalTransform::from(Transform::from_xyz(2.0, 3.0, 4.0));
        let (origin, level) = player_interaction_ray(&player, 0.0);
        let (pitched_origin, pitched) = player_interaction_ray(&player, 0.5);

        assert_eq!(origin, pitched_origin);
        assert!(level.y.abs() < 0.0001);
        assert!(pitched.y > 0.0);
    }
}

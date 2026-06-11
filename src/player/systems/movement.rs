use crate::player::components::{Player, PlayerCollider, PlayerGravity, PlayerMovement};
use bevy::prelude::*;
use crate::core::constant::player::STEP_HEIGHT;
use crate::player::systems::collision::check_collision_at;
use crate::ui::widgets::slot::SearchInputState;
use crate::voxel::registry::BlockRegistry;
use crate::world::storage::WorldStorage;

pub fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    search_state: Res<SearchInputState>,
    mut query: Query<(&mut Transform, &PlayerCollider, &PlayerMovement, &mut PlayerGravity), With<Player>>,
){
    if search_state.active { return; }
    let Some(reg) = registry else { return };
    let dt = time.delta_secs();

    for (mut transform, collider, movement, mut gravity) in &mut query{
        let half = collider.half_extents;

        // 跳跃
        if keyboard_input.just_pressed(KeyCode::Space) && gravity.is_grounded {
            // 跳跃高度计算
            gravity.velocity_y = movement.jump_force;
            // 标记着地状态，防止空中连跳
            gravity.is_grounded = false;
        }

        // 移动
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) { direction += transform.forward().as_vec3(); }
        if keyboard_input.pressed(KeyCode::KeyS) { direction -= transform.forward().as_vec3(); }
        if keyboard_input.pressed(KeyCode::KeyA) { direction -= transform.right().as_vec3(); }
        if keyboard_input.pressed(KeyCode::KeyD) { direction += transform.right().as_vec3(); }

        if direction.length() == 0.0 {
            continue;
        }

        direction.y = 0.0;
        direction = direction.normalize();

        // 处理移动速度
        let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            movement.movement_speed * movement.sprint_factor
        } else {
            movement.movement_speed
        };

        let move_delta = direction * speed * dt;

        // 分轴移动与碰撞检测
        // 处理X轴移动
        let pos = transform.translation;
        let new_pos_x = Vec3::new(pos.x + move_delta.x, pos.y, pos.z);
        if !check_collision_at(new_pos_x, half, &world_storage, &reg) {
            transform.translation.x = new_pos_x.x;
        } else if gravity.is_grounded {
            // X轴碰撞 → 尝试沿X轴爬台阶
            try_step_up(&mut transform.translation, half, move_delta.x, 0, &world_storage, &reg);
        }

        // 处理Z轴移动
        let pos = transform.translation;
        let new_pos_z = Vec3::new(pos.x, pos.y, pos.z + move_delta.z);
        if !check_collision_at(new_pos_z, half, &world_storage, &reg) {
            transform.translation.z = new_pos_z.z;
        } else if gravity.is_grounded {
            // Z轴碰撞 → 尝试沿Z轴爬台阶
            try_step_up(&mut transform.translation, half, move_delta.z, 2, &world_storage, &reg);
        }
    }
}

fn try_step_up(
    pos: &mut Vec3,
    half: Vec3,
    delta: f32,
    axis: usize,
    world_storage: &WorldStorage,
    registry: &BlockRegistry,
) {
    // 在碰撞轴上移动，同时向上抬升
    let stepped = match axis {
        0 => Vec3::new(pos.x + delta, pos.y + STEP_HEIGHT, pos.z),
        _ => Vec3::new(pos.x, pos.y + STEP_HEIGHT, pos.z + delta),
    };

    // 检测抬升后的位置是否有碰撞
    if !check_collision_at(stepped, half, world_storage, registry) {
        // 无碰撞，直接移动到台阶上
        pos.x = stepped.x;
        pos.y = stepped.y;
        pos.z = stepped.z;
    }
}
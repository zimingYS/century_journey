use crate::content::block::registry::BlockRegistry;
use crate::game::constant::player::STEP_HEIGHT;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::{
    Player, PlayerCollider, PlayerGravity, PlayerLifecycle, PlayerMovement, PlayerVelocity,
};
use crate::game::player::systems::collision::check_collision_at;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

pub fn player_movement_system(
    time: Res<Time>,
    actions: Res<PlayerActionState>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    mut query: Query<
        (
            &mut Transform,
            &PlayerCollider,
            &PlayerMovement,
            &mut PlayerGravity,
            &mut PlayerVelocity,
            &PlayerLifecycle,
        ),
        With<Player>,
    >,
) {
    let Some(reg) = registry else { return };
    let dt = time.delta_secs().min(0.05);

    for (mut transform, collider, movement, mut gravity, mut velocity, lifecycle) in &mut query {
        if !lifecycle.is_alive() {
            velocity.horizontal = Vec3::ZERO;
            continue;
        }
        let half = collider.half_extents;

        // 跳跃
        if actions.just_pressed(PlayerAction::Jump) && gravity.is_grounded {
            // 跳跃高度计算
            gravity.velocity_y = movement.jump_force;
            // 标记着地状态，防止空中连跳
            gravity.is_grounded = false;
        }

        // 移动
        let mut direction = Vec3::ZERO;
        if actions.pressed(PlayerAction::MoveForward) {
            direction += transform.forward().as_vec3();
        }
        if actions.pressed(PlayerAction::MoveBackward) {
            direction -= transform.forward().as_vec3();
        }
        if actions.pressed(PlayerAction::MoveLeft) {
            direction -= transform.right().as_vec3();
        }
        if actions.pressed(PlayerAction::MoveRight) {
            direction += transform.right().as_vec3();
        }

        direction.y = 0.0;
        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        // 处理移动速度
        let speed = if actions.pressed(PlayerAction::Sprint) {
            movement.movement_speed * movement.sprint_factor
        } else {
            movement.movement_speed
        };

        let desired_velocity = direction * speed;
        let changing_direction = direction != Vec3::ZERO
            && velocity.horizontal.length_squared() > f32::EPSILON
            && velocity.horizontal.normalize().dot(direction) < 0.8;
        let control = if gravity.is_grounded {
            if direction == Vec3::ZERO || changing_direction {
                movement.deceleration
            } else {
                movement.acceleration
            }
        } else {
            movement.acceleration * movement.air_control
        };
        velocity.horizontal =
            approach_velocity(velocity.horizontal, desired_velocity, control * dt);
        velocity.horizontal.y = 0.0;
        let move_delta = velocity.horizontal * dt;

        if move_delta.length_squared() <= f32::EPSILON {
            continue;
        }

        // 分轴移动与碰撞检测
        // 处理X轴移动
        let pos = transform.translation;
        let new_pos_x = Vec3::new(pos.x + move_delta.x, pos.y, pos.z);
        if !check_collision_at(new_pos_x, half, &world_storage, &reg) {
            transform.translation.x = new_pos_x.x;
        } else if gravity.is_grounded {
            // X 轴发生碰撞时，尝试沿 X 轴跨上台阶。
            if !try_step_up(
                &mut transform.translation,
                half,
                move_delta.x,
                0,
                &world_storage,
                &reg,
            ) {
                velocity.horizontal.x = 0.0;
            }
        } else {
            velocity.horizontal.x = 0.0;
        }

        // 处理Z轴移动
        let pos = transform.translation;
        let new_pos_z = Vec3::new(pos.x, pos.y, pos.z + move_delta.z);
        if !check_collision_at(new_pos_z, half, &world_storage, &reg) {
            transform.translation.z = new_pos_z.z;
        } else if gravity.is_grounded {
            // Z 轴发生碰撞时，尝试沿 Z 轴跨上台阶。
            if !try_step_up(
                &mut transform.translation,
                half,
                move_delta.z,
                2,
                &world_storage,
                &reg,
            ) {
                velocity.horizontal.z = 0.0;
            }
        } else {
            velocity.horizontal.z = 0.0;
        }
    }
}

fn approach_velocity(current: Vec3, target: Vec3, max_delta: f32) -> Vec3 {
    let delta = target - current;
    let distance = delta.length();
    if distance <= max_delta || distance <= f32::EPSILON {
        target
    } else {
        current + delta / distance * max_delta.max(0.0)
    }
}

fn try_step_up(
    pos: &mut Vec3,
    half: Vec3,
    delta: f32,
    axis: usize,
    world_storage: &WorldStorage,
    registry: &BlockRegistry,
) -> bool {
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
        true
    } else {
        false
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/systems/movement.rs"]
mod stage_seven_tests;

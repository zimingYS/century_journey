//! 在固定步中处理玩家水平移动、跳跃和台阶跨越。

/// 玩家可自动跨越的最大台阶高度，单位为方块。
const STEP_HEIGHT: f32 = 0.6;
use crate::content::block::registry::BlockRegistry;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::flight::components::PlayerFlight;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::PlayerLifecycle;
use crate::game::player::movement::components::{PlayerMovement, PlayerVelocity};
use crate::game::player::physics::collision::check_collision_at;
use crate::game::player::physics::components::{PlayerCollider, PlayerGravity};
use crate::game::player::survival::temperature::TemperatureExposure;
use crate::game::world::state::WorldState;
use bevy::math::Vec3;
use bevy::prelude::{Fixed, Query, Res, Time, Transform, With};

/// 移动系统在同一固定步读取碰撞体并写入变换、速度和重力状态。
#[allow(clippy::type_complexity)]
pub fn player_movement_system(
    time: Res<Time<Fixed>>,
    actions: Res<PlayerActionState>,
    registry: Option<Res<BlockRegistry>>,
    world_state: Res<WorldState>,
    mut query: Query<
        (
            &mut Transform,
            &PlayerCollider,
            &PlayerMovement,
            &mut PlayerGravity,
            &mut PlayerVelocity,
            &PlayerLifecycle,
            &PlayerFlight,
            Option<&TemperatureExposure>,
        ),
        With<Player>,
    >,
) {
    let Some(reg) = registry else { return };
    let dt = time.delta_secs().min(0.05);

    for (
        mut transform,
        collider,
        movement,
        mut gravity,
        mut velocity,
        lifecycle,
        flight,
        temperature,
    ) in &mut query
    {
        if !lifecycle.is_alive() {
            velocity.horizontal = Vec3::ZERO;
            continue;
        }
        let half = collider.half_extents;

        // 跳跃
        if !flight.enabled && actions.just_pressed(PlayerAction::Jump) && gravity.is_grounded {
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

        // 处理移动速度；温度惩罚按倍率缩放最终速度。
        let base_speed = if flight.enabled {
            // 飞行加速
            if actions.pressed(PlayerAction::Sprint) {
                flight.fly_speed * movement.sprint_factor * 1.5
            } else {
                flight.fly_speed
            }
        } else if actions.pressed(PlayerAction::Sprint) {
            // 移动加速
            movement.movement_speed * movement.sprint_factor
        } else {
            movement.movement_speed
        };
        let speed = base_speed * temperature.map_or(1.0, |exposure| exposure.speed_multiplier);

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
        if flight.enabled {
            // 飞行：水平速度直接取目标速度，瞬时响应，与垂直速度保持一致。
            velocity.horizontal = desired_velocity;
        } else {
            velocity.horizontal =
                approach_velocity(velocity.horizontal, desired_velocity, control * dt);
        }
        velocity.horizontal.y = 0.0;
        let move_delta = velocity.horizontal * dt;

        if move_delta.length_squared() <= f32::EPSILON {
            continue;
        }

        // 分轴移动与碰撞检测
        // 处理X轴移动
        let pos = transform.translation;
        let new_pos_x = Vec3::new(pos.x + move_delta.x, pos.y, pos.z);
        if !check_collision_at(new_pos_x, half, &world_state, &reg) {
            transform.translation.x = new_pos_x.x;
        } else if gravity.is_grounded && !flight.enabled {
            // X 轴发生碰撞时，尝试沿 X 轴跨上台阶。
            if !try_step_up(
                &mut transform.translation,
                half,
                move_delta.x,
                0,
                &world_state,
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
        if !check_collision_at(new_pos_z, half, &world_state, &reg) {
            transform.translation.z = new_pos_z.z;
        } else if gravity.is_grounded && !flight.enabled {
            // Z 轴发生碰撞时，尝试沿 Z 轴跨上台阶。
            if !try_step_up(
                &mut transform.translation,
                half,
                move_delta.z,
                2,
                &world_state,
                &reg,
            ) {
                velocity.horizontal.z = 0.0;
            }
        } else {
            velocity.horizontal.z = 0.0;
        }
    }
}

/// 以不超过 `max_delta` 的变化量把当前水平速度逼近目标速度。
pub fn approach_velocity(current: Vec3, target: Vec3, max_delta: f32) -> Vec3 {
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
    world_storage: &WorldState,
    registry: &BlockRegistry,
) -> bool {
    // 台阶通过性必须同时验证水平目标和抬升后的完整碰撞箱。
    let stepped = match axis {
        0 => Vec3::new(pos.x + delta, pos.y + STEP_HEIGHT, pos.z),
        _ => Vec3::new(pos.x, pos.y + STEP_HEIGHT, pos.z + delta),
    };

    if !check_collision_at(stepped, half, world_storage, registry) {
        pos.x = stepped.x;
        pos.y = stepped.y;
        pos.z = stepped.z;
        true
    } else {
        false
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/movement/system.rs"]
mod tests;

use crate::content::block::registry::BlockRegistry;
use crate::game::constant::player::{GRAVITY, MAX_FALL_SPEED};
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::player::components::{Player, PlayerCollider, PlayerGravity, PlayerLifecycle};
use crate::game::player::events::{DamageEvent, DamageSource};
use crate::game::player::systems::collision::{
    check_collision_at, find_safe_position, is_grounded_at,
};
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

/// 重力系统
pub fn player_gravity_system(
    time: Res<Time>,
    registry: Option<Res<BlockRegistry>>,
    gamemode: Res<PlayerGameMode>,
    world_storage: Res<WorldStorage>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &PlayerCollider,
            &mut PlayerGravity,
            &PlayerLifecycle,
        ),
        With<Player>,
    >,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    let Some(reg) = registry else { return };
    let dt = time.delta_secs();

    for (entity, mut transform, collider, mut gravity, lifecycle) in &mut query {
        if !lifecycle.is_alive() {
            gravity.velocity_y = 0.0;
            gravity.fall_distance = 0.0;
            continue;
        }
        let half = collider.half_extents;
        let pos = transform.translation;

        // 卡在方块时自动寻找安全位置
        if check_collision_at(pos, half, &world_storage, &reg) {
            if let Some(safe_pos) = find_safe_position(pos, half, &world_storage, &reg) {
                transform.translation = safe_pos;
                gravity.velocity_y = 0.0;
                gravity.is_grounded = true;
                gravity.fall_distance = 0.0;
            }
            continue;
        }

        // 着地且没跳则只检查是否还在地面
        if gravity.is_grounded && gravity.velocity_y <= 0.0 {
            gravity.velocity_y = 0.0;
            if is_grounded_at(pos, half, &world_storage, &reg) {
                // 还在地面，保持状态
                gravity.fall_distance = 0.0;
                continue;
            }
            // 脚下没地则开始下落
            gravity.is_grounded = false;
        }

        // 应用重力
        gravity.velocity_y += GRAVITY * dt;
        if gravity.velocity_y < MAX_FALL_SPEED {
            gravity.velocity_y = MAX_FALL_SPEED;
        }

        // 计算垂直移动
        let move_y = gravity.velocity_y * dt;
        if move_y < 0.0 {
            gravity.fall_distance += -move_y;
        }
        let new_pos = Vec3::new(pos.x, pos.y + move_y, pos.z);

        // 先检测再移动
        if !check_collision_at(new_pos, half, &world_storage, &reg) {
            transform.translation.y = new_pos.y;
            gravity.is_grounded = false;
        } else {
            // 碰撞了，不移动
            if gravity.velocity_y < 0.0 {
                // 向下碰撞 → 着地
                gravity.is_grounded = true;
                let damage = fall_damage_from_distance(gravity.fall_distance);
                if gamemode.is_survival() && damage > 0.0 {
                    damage_writer.write(DamageEvent {
                        target: entity,
                        amount: damage,
                        source: DamageSource::Fall,
                    });
                }
            } else {
                // 向上碰撞 → 撞顶
                gravity.is_grounded = false;
            }
            gravity.velocity_y = 0.0;
            gravity.fall_distance = 0.0;
        }
    }
}

/// 前三个方块不造成伤害，之后每多下落一个方块造成一点伤害。
pub fn fall_damage_from_distance(distance: f32) -> f32 {
    (distance - 3.0).floor().max(0.0)
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/systems/gravity.rs"]
mod stage_seven_tests;

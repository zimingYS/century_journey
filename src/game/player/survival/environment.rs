//! 从玩家位置采样环境介质，并更新水下和氧气状态。

use crate::content::block::registry::BlockRegistry;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::PlayerLifecycle;
use crate::game::player::survival::events::{DamageEvent, DamageSource};
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;
use bevy::math::Vec3;
use bevy::prelude::*;

/// 环境暴露计时，集中保存溺水和周期环境伤害的状态。
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentExposure {
    /// 头部浸没时剩余的可呼吸秒数。
    pub remaining_air: f32,
    /// 下一次周期环境伤害生效前的固定步倒计时。
    pub damage_cooldown: f32,
}

impl Default for EnvironmentExposure {
    fn default() -> Self {
        Self {
            remaining_air: 10.0,
            damage_cooldown: 0.0,
        }
    }
}

const MAX_AIR_SECONDS: f32 = 10.0;
const VOID_Y: f32 = -32.0;

/// 检测头部水体、火焰方块与虚空，转换为统一环境伤害。
pub fn environment_damage_system(
    time: Res<Time<Fixed>>,
    registry: Option<Res<BlockRegistry>>,
    world_state: Res<WorldState>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &mut EnvironmentExposure,
            &PlayerLifecycle,
        ),
        With<Player>,
    >,
    mut writer: MessageWriter<DamageEvent>,
) {
    let Some(registry) = registry else {
        return;
    };
    let dt = time.delta_secs();

    for (entity, transform, mut exposure, lifecycle) in &mut query {
        if !lifecycle.is_alive() {
            continue;
        }
        exposure.damage_cooldown = (exposure.damage_cooldown - dt).max(0.0);

        let head = transform.translation + Vec3::Y * 0.8;
        let block_pos = head.floor().as_ivec3();
        let block_id = get_voxel_at_world(block_pos, &world_state);
        let block_path = registry
            .get_identifier_by_id(block_id)
            .map(|identifier| identifier.path());
        let submerged = block_path == Some("water");

        if submerged {
            exposure.remaining_air = (exposure.remaining_air - dt).max(0.0);
        } else {
            exposure.remaining_air = (exposure.remaining_air + dt * 4.0).min(MAX_AIR_SECONDS);
        }

        let damage = if transform.translation.y < VOID_Y {
            Some((DamageSource::Generic, 4.0, 0.5))
        } else if matches!(block_path, Some("fire" | "lava")) {
            Some((DamageSource::Fire, 2.0, 1.0))
        } else if submerged && exposure.remaining_air <= 0.0 {
            Some((DamageSource::Drowning, 1.0, 1.0))
        } else {
            None
        };

        if let Some((source, amount, cooldown)) = damage
            && exposure.damage_cooldown <= 0.0
        {
            writer.write(DamageEvent {
                target: entity,
                amount,
                source,
            });
            exposure.damage_cooldown = cooldown;
        }
    }
}

use crate::content::block::registry::BlockRegistry;
use crate::game::player::components::{EnvironmentExposure, Player, PlayerLifecycle};
use crate::game::player::events::{DamageEvent, DamageSource};
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

const MAX_AIR_SECONDS: f32 = 10.0;
const VOID_Y: f32 = -32.0;

/// 检测头部水体、火焰方块与虚空，转换为统一环境伤害。
pub fn environment_damage_system(
    time: Res<Time>,
    registry: Option<Res<BlockRegistry>>,
    storage: Res<WorldStorage>,
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
        let block_id = get_voxel_at_world(block_pos, &storage);
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

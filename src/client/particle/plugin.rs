//! 组装客户端粒子发射、更新与回收系统。

use bevy::prelude::*;

use crate::client::particle::systems::{
    spawn_action_particles_system, spawn_block_particles_system, update_feedback_particles_system,
};
use crate::client::particle::types::ParticleVisuals;
use crate::shared::states::AppState;

/// 注册仅影响视觉的粒子发射、更新和回收系统。
pub struct ClientParticlePlugin;

impl Plugin for ClientParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleVisuals>().add_systems(
            Update,
            (
                spawn_block_particles_system,
                spawn_action_particles_system,
                update_feedback_particles_system,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

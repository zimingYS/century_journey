//! 组装水面动态材质与水下滤镜表现系统。

use bevy::prelude::*;

use crate::client::water::{material, systems};
use crate::shared::states::AppState;

/// 组装水面动态材质与水下滤镜表现系统。
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<material::WaterMaterial>::default())
            .init_resource::<systems::UnderwaterState>()
            .add_systems(Startup, systems::spawn_underwater_overlay_system)
            .add_systems(
                Update,
                (
                    systems::water_flow_animation_system,
                    systems::underwater_detect_system,
                    systems::underwater_filter_system
                        .after(crate::client::sky::systems::atmosphere_system)
                        .after(systems::underwater_detect_system),
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnExit(AppState::InGame),
                systems::reset_underwater_state_system,
            );
    }
}

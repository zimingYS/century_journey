//! 组装云层资源、实体与渲染帧表现系统。

use bevy::prelude::*;

use crate::client::sky::cloud::{components, systems, weather_adapter};
use crate::shared::states::AppState;

/// 组装云层资源、实体与渲染帧表现系统。
pub struct CloudPlugin;

impl Plugin for CloudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::CloudRuntime>()
            .init_resource::<components::CloudWeatherState>()
            .add_systems(
                OnEnter(AppState::InGame),
                systems::setup_cloud_system
                    .after(crate::content::lifecycle::ContentReloadSet::Consumers),
            )
            .add_systems(OnExit(AppState::InGame), systems::cleanup_cloud_system)
            .add_systems(
                Update,
                (
                    weather_adapter::sync_weather_to_cloud_system,
                    systems::cloud_drift_system,
                    systems::cloud_tint_system,
                    systems::cloud_patch_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

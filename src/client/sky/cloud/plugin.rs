//! 组装云层资源、实体与渲染帧表现系统。
//!
//! V2 (08-18)：当前注册 voxel 块状云路径（ARTShade 风格几何云）。原
//! raymarching 系统（`material.rs`、`systems::setup_cloud_system`、
//! `cloud_volume_update_system` 等）的代码保留在仓库中但未注册，可作为
//! 未来切换回体积云的备份入口。

use super::weather_adapter;
use crate::client::sky::cloud::material::CloudVolumeMaterial;
use crate::client::sky::cloud::systems::{
    CloudRuntime, cleanup_cloud_system, cloud_patch_system, cloud_tint_system,
    cloud_volume_update_system, setup_cloud_system,
};
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

/// 组装云层资源、实体与渲染帧表现系统。
pub struct CloudPlugin;

impl Plugin for CloudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CloudRuntime>()
            .init_resource::<super::components::CloudWeatherState>()
            .add_plugins(MaterialPlugin::<CloudVolumeMaterial>::default())
            .add_systems(
                OnEnter(AppState::InGame),
                setup_cloud_system.after(crate::content::lifecycle::ContentReloadSet::Consumers),
            )
            .add_systems(OnExit(AppState::InGame), cleanup_cloud_system)
            .add_systems(
                Update,
                (
                    cloud_volume_update_system,
                    cloud_tint_system,
                    cloud_patch_system,
                    weather_adapter::sync_weather_to_cloud_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

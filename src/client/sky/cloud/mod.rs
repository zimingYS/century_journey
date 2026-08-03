//! 组织多层动态云的实体创建与每帧表现更新。
//!
//! 云是纯表现层功能：读取 Content 编译的云定义与客户端时间快照，在渲染帧
//! 驱动云层漂移、昼夜染色与近景云片朝向。不进入 FixedUpdate，不参与权威模拟。

mod components;
mod constants;
mod systems;
mod texture;

use bevy::prelude::*;

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
                    systems::cloud_drift_system,
                    systems::cloud_tint_system,
                    systems::cloud_patch_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// 供单元测试与天气适配器使用的类型入口。
#[doc(hidden)]
pub use components::{CloudLayer, CloudPatch, CloudWeatherState};
#[doc(hidden)]
pub use constants::CLOUD_TEXTURE_SIZE;
#[doc(hidden)]
pub use texture::generate_cloud_texture;

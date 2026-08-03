//! 组织水面动态材质与水下视觉滤镜。
//!
//! 本模块是纯表现层：水面材质在 GPU 中计算波法线、深度渐变和岸线泡沫，
//! 水下滤镜检测玩家头部浸没并投影到相机色调、雾效和曝光。
//! 不参与 FixedUpdate，不改变任何权威游戏规则。

mod components;
mod constants;
mod material;
mod systems;

use bevy::prelude::*;

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

#[doc(hidden)]
pub use components::UnderwaterOverlay;
#[doc(hidden)]
pub use constants::{WATER_FLOW_SPEED, WATER_FLOW_TILE};
#[doc(hidden)]
pub use material::{WaterMaterial, WaterMaterialExtension};
#[doc(hidden)]
pub use systems::{
    compute_underwater_alpha, underwater_depth_step, water_depth_factor, water_flow_offset,
};

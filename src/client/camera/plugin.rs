//! 组装客户端相机创建、输入和视角切换系统。

use bevy::prelude::*;

use crate::client::camera::systems::{
    camera_perspective_sync_system, player_look_system, setup_player_camera_system,
    toggle_perspective_system,
};
use crate::client::input::InputSet;
use crate::shared::states::AppState;

/// 注册客户端相机创建、输入和视角切换系统。
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            player_look_system
                .after(InputSet::ResolveContext)
                .before(InputSet::CollectActions)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                (toggle_perspective_system, camera_perspective_sync_system).chain(),
                setup_player_camera_system,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

//! 管理由权威模拟状态派生出的客户端连续表现状态。

mod time;

pub use time::{TimeOfDay, TimePhase};

use bevy::prelude::*;

use crate::shared::states::AppState;

/// 组装仅供客户端读取的表现插值资源。
pub struct ClientPresentationPlugin;

impl Plugin for ClientPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeOfDay>().add_systems(
            PreUpdate,
            time::update_visual_time.run_if(in_state(AppState::InGame)),
        );
    }
}

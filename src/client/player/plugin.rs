//! 组装本地玩家模型、相机可见层和手持物表现。

use bevy::prelude::*;

use crate::client::camera::CameraPlugin;
use crate::client::player::full_body::FullBodyFirstPersonPlugin;
use crate::client::player::model::PlayerModelPlugin;
use crate::client::player::systems::{
    attach_local_player_presentation_system, first_person_visibility_system,
};
use crate::game::player::lifecycle::spawn::PlayerStartupSet;

/// 组装本地玩家模型、相机可见层和手持物表现。
pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerModelPlugin)
            .add_plugins(FullBodyFirstPersonPlugin)
            .add_plugins(CameraPlugin)
            .add_systems(
                Startup,
                attach_local_player_presentation_system.after(PlayerStartupSet::Authority),
            )
            .add_systems(Update, first_person_visibility_system);
    }
}

//! 组装客户端反馈音效的播放系统。

use bevy::prelude::*;

use crate::client::sound::resources::{AmbientClock, FeedbackAudioAssets, SoundSequence};
use crate::client::sound::systems::{
    ambient_sound_system, animation_marker_sound_system, block_sound_system, dialog_sound_system,
    footstep_sound_system, game_ready_sound_system, inventory_feedback_sound_system,
    loading_sound_system, ui_interaction_sound_system, ui_navigation_sound_system,
};
use crate::shared::states::AppState;

/// 注册环境、方块和玩家反馈音效的客户端播放系统。
pub struct ClientSoundPlugin;

impl Plugin for ClientSoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeedbackAudioAssets>()
            .init_resource::<SoundSequence>()
            .init_resource::<AmbientClock>()
            .add_systems(
                Update,
                (
                    ui_interaction_sound_system,
                    ui_navigation_sound_system,
                    dialog_sound_system,
                    inventory_feedback_sound_system,
                ),
            )
            .add_systems(
                Update,
                (
                    block_sound_system,
                    animation_marker_sound_system,
                    footstep_sound_system,
                    ambient_sound_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::Loading), loading_sound_system)
            .add_systems(OnEnter(AppState::WorldLoading), loading_sound_system)
            .add_systems(OnEnter(AppState::InGame), game_ready_sound_system);
    }
}

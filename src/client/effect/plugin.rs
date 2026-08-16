//! 组装方块高亮等客户端视觉反馈效果。

use bevy::prelude::*;

use crate::client::effect::resources::{DamageFeedback, NoticeFeedback};
use crate::client::effect::systems::{
    camera_shake_system, clear_feedback_on_exit_system, draw_break_cracks_system,
    receive_damage_feedback_system, receive_notice_feedback_system, spawn_feedback_ui_system,
    update_feedback_ui_system,
};
use crate::shared::states::AppState;

/// 组装方块高亮等客户端视觉反馈效果。
pub struct ClientEffectPlugin;

impl Plugin for ClientEffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DamageFeedback>()
            .init_resource::<NoticeFeedback>()
            .add_systems(
                Startup,
                spawn_feedback_ui_system
                    .after(crate::client::ui::resources::ui_font::load_ui_font_system),
            )
            .add_systems(
                Update,
                (
                    receive_damage_feedback_system,
                    receive_notice_feedback_system,
                    update_feedback_ui_system,
                    draw_break_cracks_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), clear_feedback_on_exit_system)
            .add_systems(
                PostUpdate,
                camera_shake_system
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (super::voxel_highlight::draw_voxel_highlight_system)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

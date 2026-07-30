//! 注册通用控件主题、滚动和鼠标携带物品的表现系统。

use bevy::prelude::*;

use super::{common, drag};
use crate::client::ui::{resources, theme};

/// 组装可被多个屏幕复用的 UI 控件系统。
pub struct UiWidgetsPlugin;

impl Plugin for UiWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                theme::scale::sync_ui_scale_system,
                resources::frame_assets::apply_ui_frame_system,
                common::themed_control_interaction_system,
                common::scroll_area_wheel_system,
            ),
        )
        .add_systems(
            Update,
            (
                drag::cursor_follow_system,
                drag::cursor_visibility_system,
                drag::cursor_texture_system,
            ),
        );
    }
}

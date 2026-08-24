//! 组装调试浮层的资源与系统。

use super::components::DebugOverlayState;
use super::systems::{
    spawn_debug_overlay_system, sync_debug_overlay_visibility_system, toggle_debug_overlay_system,
    update_debug_overlay_text_system,
};
use crate::client::ui::resources::ui_font::load_ui_font_system;
use bevy::prelude::*;

/// F3 调试浮层插件：Startup 生成常驻节点，运行期按开关显隐。
pub struct DebugOverlayPlugin;

impl Plugin for DebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOverlayState>()
            .add_systems(
                Startup,
                spawn_debug_overlay_system.after(load_ui_font_system),
            )
            .add_systems(
                Update,
                (
                    toggle_debug_overlay_system,
                    sync_debug_overlay_visibility_system,
                    update_debug_overlay_text_system,
                ),
            );
    }
}

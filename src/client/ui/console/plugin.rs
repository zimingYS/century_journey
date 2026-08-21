//! 组装控制台的资源、消息与系统。

use super::components::{ConsoleLineSubmitted, ConsoleState};
use crate::client::ui::console::systems::{
    push_console_line_system, spawn_console_system, sync_console_open_system,
    update_console_message_system,
};
use crate::client::ui::resources::ui_font::load_ui_font_system;
use bevy::prelude::*;

/// 组装控制台 UI 资源、消息和系统。
pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleState>()
            .add_message::<ConsoleLineSubmitted>()
            .add_systems(Startup, spawn_console_system.after(load_ui_font_system))
            .add_systems(
                Update,
                (
                    push_console_line_system,
                    sync_console_open_system,
                    update_console_message_system,
                ),
            );
    }
}

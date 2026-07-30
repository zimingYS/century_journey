//! 根据已解析的输入上下文同步系统光标的捕获与可见状态。

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::shared::states::InputContextState;

/// 在帧末依据最终输入上下文同步主窗口光标。
pub(super) fn sync_cursor_state_system(
    context: Res<InputContextState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor_query.single_mut() else {
        return;
    };
    let gameplay = context.active().allows_gameplay();
    cursor.visible = !gameplay;
    cursor.grab_mode = if gameplay {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

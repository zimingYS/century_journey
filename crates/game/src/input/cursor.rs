use crate::input::intent::ToggleCursorCapture;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

/// 启动时锁定并隐藏鼠标游标
pub fn lock_cursor_on_start(mut cursor: Query<&mut CursorOptions>) {
    for mut options in &mut cursor {
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }
}

/// 处理鼠标游标捕获状态切换
pub fn toggle_cursor_capture(
    mut messages: MessageReader<ToggleCursorCapture>,
    mut cursor: Query<&mut CursorOptions>,
) {
    for _ in messages.read() {
        for mut options in &mut cursor {
            match options.grab_mode {
                CursorGrabMode::Locked => {
                    options.grab_mode = CursorGrabMode::None;
                    options.visible = true;
                }
                CursorGrabMode::None => {
                    options.grab_mode = CursorGrabMode::Locked;
                    options.visible = false;
                }
                _ => {
                    options.grab_mode = CursorGrabMode::Locked;
                    options.visible = false;
                }
            }
        }
    }
}

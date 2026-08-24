//! 键位重映射的按键捕获：等待玩家按下新按键并生成绑定命令。

use bevy::prelude::*;

use crate::app::flow::FlowCommand;
use crate::app::settings::{BindingKey, KeyAction};

/// 当前正在等待新按键的动作；None 表示不在捕获状态。
///
/// 由设置界面的键位按钮进入，本系统消费一次按键后自动退出。
#[derive(Resource, Debug, Default)]
pub struct RebindCapture {
    pub listening: Option<KeyAction>,
}

/// 捕获重绑定按键：Esc 取消、Backspace 解除绑定、其他任意键生效为新绑定。
///
/// 必须先于界面输入系统运行，并清空已消费按键的按下边沿，
/// 防止捕获的按键同帧泄漏为背包开关等界面命令。
pub(super) fn capture_rebind_input_system(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut capture: ResMut<RebindCapture>,
    mut flow: MessageWriter<FlowCommand>,
) {
    let Some(action) = capture.listening else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        keyboard.clear_just_pressed(KeyCode::Escape);
        capture.listening = None;
        return;
    }
    if keyboard.just_pressed(KeyCode::Backspace) {
        keyboard.clear_just_pressed(KeyCode::Backspace);
        flow.write(FlowCommand::RebindKey(action, None));
        capture.listening = None;
        return;
    }

    let key = keyboard.get_just_pressed().next().copied();
    let button = mouse.get_just_pressed().next().copied();
    let binding = match (key, button) {
        (Some(code), _) => Some(BindingKey::Key(code)),
        (None, Some(mouse_button)) => Some(BindingKey::Mouse(mouse_button)),
        (None, None) => None,
    };
    let Some(binding) = binding else {
        return;
    };
    // 清空全部按下边沿：被绑定键本帧已被消费，不应再触发其他输入系统。
    for code in keyboard.get_just_pressed().copied().collect::<Vec<_>>() {
        keyboard.clear_just_pressed(code);
    }
    for mouse_button in mouse.get_just_pressed().copied().collect::<Vec<_>>() {
        mouse.clear_just_pressed(mouse_button);
    }
    flow.write(FlowCommand::RebindKey(action, Some(binding)));
    capture.listening = None;
}

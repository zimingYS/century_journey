//! 采集控制台开合与提交按键，管理输入焦点。

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::text::{EditableText, TextEdit};

use crate::client::ui::console::components::{ConsoleInput, ConsoleLineSubmitted, ConsoleState};
use crate::shared::states::app_state::AppState;

/// 该系统是控制台输入的单一装配点，参数保持显式以便审查每个可写资源和消息出口。
#[allow(clippy::too_many_arguments)]
pub(super) fn console_keyboard_system(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut console: ResMut<ConsoleState>,
    mut focus: ResMut<InputFocus>,
    input_query: Query<Entity, With<ConsoleInput>>,
    mut input_visibility_query: Query<&mut Visibility, With<ConsoleInput>>,
    mut editable_query: Query<&mut EditableText, With<ConsoleInput>>,
    mut lines: MessageWriter<ConsoleLineSubmitted>,
) {
    if *app_state.get() != AppState::InGame {
        return;
    }
    let Ok(input_entity) = input_query.single() else {
        return;
    };

    if !console.open {
        // 关闭状态：Enter 打开输入框。
        if keyboard.just_pressed(KeyCode::Enter) {
            keyboard.clear_just_pressed(KeyCode::Enter);
            console.open = true;
            if let Ok(mut editable) = editable_query.single_mut() {
                editable.clear();
            }
            focus.set(input_entity, FocusCause::Navigated);
            if let Ok(mut vis) = input_visibility_query.single_mut() {
                *vis = Visibility::Visible;
            }
        }
        return;
    }

    // 打开状态。
    // ↑/↓ 翻阅输入历史：IME 组合中方向键留给候选词导航，不拦截。
    let browse_older = keyboard.just_pressed(KeyCode::ArrowUp);
    let browse_newer = keyboard.just_pressed(KeyCode::ArrowDown);
    if browse_older || browse_newer {
        keyboard.clear_just_pressed(KeyCode::ArrowUp);
        keyboard.clear_just_pressed(KeyCode::ArrowDown);
        if let Ok(mut editable) = editable_query.single_mut()
            && !editable.is_composing()
        {
            let current = editable.value().to_string();
            let replacement = if browse_older {
                console.input_history.browse_older(&current)
            } else {
                console.input_history.browse_newer()
            };
            if let Some(text) = replacement {
                replace_input_line(&mut editable, text);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        keyboard.clear_just_pressed(KeyCode::Escape);
        close_console(
            &mut console,
            &mut focus,
            &mut editable_query,
            &mut input_visibility_query,
        );
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        keyboard.clear_just_pressed(KeyCode::Enter);
        // IME 组合中（如拼音未上屏）不提交，让组合先完成。
        if let Ok(editable) = editable_query.single()
            && !editable.is_composing()
        {
            let text = editable.value().to_string();
            console.input_history.record(&text);
            lines.write(ConsoleLineSubmitted { text });
        }
        // 发送后立即关闭输入框（聊天框生命周期：Enter 发送同时输入框关闭）。
        close_console(
            &mut console,
            &mut focus,
            &mut editable_query,
            &mut input_visibility_query,
        );
    }
}

/// 整行替换输入框内容：丢弃未应用的编辑动作并把光标移到行尾。
fn replace_input_line(editable: &mut EditableText, text: &str) {
    editable.clear();
    editable.editor_mut().set_text(text);
    editable.queue_edit(TextEdit::TextEnd(false));
}

/// 关闭输入框：只影响输入框的可见性与焦点，不触碰历史消息区（root/history 始终显示）。
fn close_console(
    console: &mut ConsoleState,
    focus: &mut InputFocus,
    editable_query: &mut Query<&mut EditableText, With<ConsoleInput>>,
    input_visibility_query: &mut Query<&mut Visibility, With<ConsoleInput>>,
) {
    console.open = false;
    console.input_history.reset_browsing();
    focus.clear();
    if let Ok(mut editable) = editable_query.single_mut() {
        editable.clear();
    }
    if let Ok(mut vis) = input_visibility_query.single_mut() {
        *vis = Visibility::Hidden;
    }
}

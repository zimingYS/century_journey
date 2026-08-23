//! 控制台的 UI 构造与历史回显系统。

use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};

use super::components::{
    ConsoleHint, ConsoleHistory, ConsoleInput, ConsoleLineSubmitted, ConsoleMessage, ConsoleRoot,
    ConsoleState,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::game::command::components::{CommandOutput, GameCommandSubmitted};
use crate::game::command::suggest::completions;

const CONSOLE_VISIBLE_SECONDS: f32 = 5.0;
/// 单条消息淡出过渡时长（秒）。
const CONSOLE_FADE_SECONDS: f32 = 1.0;

/// 构造控制台 UI 树：历史区在上，输入框在下，默认隐藏。
pub fn spawn_console_system(mut commands: Commands, ui_font: Res<UiFont>, theme: Res<UiTheme>) {
    commands
        .spawn((
            ConsoleRoot,
            Name::new("Console"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                bottom: Val::Percent(12.0),
                width: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            GlobalZIndex(6000),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn((
                ConsoleHistory,
                Visibility::Visible,
                Name::new("ConsoleHistory"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexEnd,
                    row_gap: Val::Px(2.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
            ));
            root.spawn((
                ConsoleHint,
                Visibility::Hidden,
                Name::new("ConsoleHint"),
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(theme.text_secondary),
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
            ));
            root.spawn((
                ConsoleInput,
                Visibility::Hidden,
                Name::new("ConsoleInput"),
                EditableText {
                    visible_width: Some(40.0),
                    max_characters: Some(256),
                    allow_newlines: false,
                    ..default()
                },
                TextCursorStyle {
                    color: theme.text_primary,
                    selection_color: theme.border_selected,
                    unfocused_selection_color: theme.border_hover,
                    selected_text_color: Some(Color::BLACK),
                },
                TextLayout::no_wrap(),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(theme.text_primary),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(32.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 0.8)),
            ));
        });
}

/// 在历史容器下 spawn 一条带淡出状态机的可见消息。
fn spawn_console_message(
    history: Entity,
    commands: &mut Commands,
    text: String,
    ui_font: &UiFont,
    theme: &UiTheme,
) {
    commands.entity(history).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(theme.text_primary),
            ConsoleMessage {
                timer: Timer::from_seconds(CONSOLE_VISIBLE_SECONDS, TimerMode::Once),
                fading: false,
                fade_timer: Timer::from_seconds(CONSOLE_FADE_SECONDS, TimerMode::Once),
            },
        ));
    });
}

/// 消费提交行：先写入持久历史，再 spawn 一条可见的表现消息。
///
/// 以 '/' 开头的行是指令，交给指令系统处理，不进入聊天历史。
pub fn push_console_line_system(
    mut lines: MessageReader<ConsoleLineSubmitted>,
    mut console: ResMut<ConsoleState>,
    history_query: Query<Entity, With<ConsoleHistory>>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    mut commands: Commands,
) {
    let Ok(history) = history_query.single() else {
        return;
    };
    for line in lines.read() {
        if line.text.trim().is_empty() {
            continue;
        }
        if line.text.trim_start().starts_with('/') {
            continue;
        }
        // 数据层：永久记录，UI 隐藏不删除。
        console.history.push(line.text.clone());
        // 表现层：spawn 一条可见消息，带淡出状态机。
        spawn_console_message(
            history,
            &mut commands,
            format!("> {}", line.text),
            &ui_font,
            &theme,
        );
    }
}

/// 把以 '/' 开头的控制台行转发为 Game 层指令消息（剥离前导 '/'）。
pub fn forward_command_system(
    mut lines: MessageReader<ConsoleLineSubmitted>,
    mut game_commands: MessageWriter<GameCommandSubmitted>,
) {
    for line in lines.read() {
        let Some(raw) = line.text.trim_start().strip_prefix('/') else {
            continue;
        };
        if raw.trim().is_empty() {
            continue;
        }
        game_commands.write(GameCommandSubmitted {
            raw: raw.to_owned(),
        });
    }
}

/// 把指令反馈渲染为控制台可见消息（复用淡出状态机，不进入聊天历史）。
pub fn push_command_output_system(
    mut outputs: MessageReader<CommandOutput>,
    history_query: Query<Entity, With<ConsoleHistory>>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    mut commands: Commands,
) {
    let Ok(history) = history_query.single() else {
        return;
    };
    for output in outputs.read() {
        spawn_console_message(
            history,
            &mut commands,
            output.text.clone(),
            &ui_font,
            &theme,
        );
    }
}

/// 每帧刷新指令提示：指令行显示补全候选与用法，其他情况隐藏。
///
/// 位于历史区与输入框之间；根节点锚定屏幕下方，提示出现时只会把历史区
/// 向上推移，输入框位置保持不动。
pub fn update_console_hint_system(
    console: Res<ConsoleState>,
    editable_query: Query<&EditableText, With<ConsoleInput>>,
    mut hint_query: Query<(&mut Text, &mut Visibility), With<ConsoleHint>>,
) {
    let Ok((mut text, mut visibility)) = hint_query.single_mut() else {
        return;
    };
    let line = if console.open {
        editable_query
            .single()
            .ok()
            .map(|editable| editable.value().to_string())
    } else {
        None
    };
    let Some(line) = line else {
        *visibility = Visibility::Hidden;
        return;
    };
    if !line.trim_start().starts_with('/') {
        *visibility = Visibility::Hidden;
        return;
    }
    let suggestions = completions(&line);
    let mut sections: Vec<String> = suggestions
        .lines
        .iter()
        .map(|candidate| candidate.trim_end().to_owned())
        .collect();
    if let Some(usage) = suggestions.usage {
        sections.push(usage.to_owned());
    }
    let joined = sections.join("\n");
    if joined.is_empty() {
        *visibility = Visibility::Hidden;
        return;
    }
    if text.0 != joined {
        text.0 = joined;
    }
    *visibility = Visibility::Visible;
}

/// 输入框开合边沿：打开重置所有消息的淡出状态；关闭时已过期隐藏、未过期继续显示。
pub fn sync_console_open_system(
    console: Res<ConsoleState>,
    mut previous: Local<bool>,
    mut input_query: Query<&mut Visibility, (With<ConsoleInput>, Without<ConsoleMessage>)>,
    mut message_query: Query<(&mut ConsoleMessage, &mut TextColor, &mut Visibility)>,
) {
    let opened = console.open && !*previous;
    let closed = !console.open && *previous;
    *previous = console.open;

    if opened {
        // 打开：重置所有消息的淡出状态（timer 不重置），输入框可见
        for (mut message, mut color, mut visibility) in &mut message_query {
            message.fading = false;
            message.fade_timer.reset();
            color.0.set_alpha(1.0);
            *visibility = Visibility::Visible;
        }
        if let Ok(mut vis) = input_query.single_mut() {
            *vis = Visibility::Visible;
        }
    } else if closed {
        // 关闭：输入框隐藏；已过期的消息隐藏，未过期的继续显示
        if let Ok(mut vis) = input_query.single_mut() {
            *vis = Visibility::Hidden;
        }
        for (message, _, mut visibility) in &mut message_query {
            *visibility = if message.fading {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}

/// 推进每条消息的独立计时：未过期 tick 显示计时并进入淡出，淡出中递减 alpha 并最终 Hidden。
pub fn update_console_message_system(
    time: Res<Time>,
    console: Res<ConsoleState>,
    mut query: Query<(&mut ConsoleMessage, &mut TextColor, &mut Visibility)>,
) {
    // 输入框开启时，历史文本持续显示，不推进淡出计时。
    if console.open {
        return;
    }

    for (mut message, mut color, mut visibility) in &mut query {
        if !message.fading {
            message.timer.tick(time.delta());
            if message.timer.is_finished() {
                message.fading = true;
            }
        } else {
            message.fade_timer.tick(time.delta());
            color.0.set_alpha(1.0 - message.fade_timer.fraction());
            if message.fade_timer.is_finished() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
